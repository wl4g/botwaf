// SPDX-License-Identifier: GNU GENERAL PUBLIC LICENSE Version 3
//
// Copyleft (c) 2024 James Wong. This file is part of James Wong.
// is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// James Wong is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with James Wong.  If not, see <https://www.gnu.org/licenses/>.
//
// IMPORTANT: Any software that fully or partially contains or uses materials
// covered by this license must also be released under the GNU GPL license.
// This includes modifications and derived works.

use crate::forwarder_base::IForwarder;
use anyhow::{Context, Result};
use async_trait::async_trait;
use axum::{body::Body, response::Response};
use botwaf_server::config::config;
use botwaf_types::forwarder::HttpIncomingRequest;
use common_telemetry::{debug, info};
use hyper::{header, Method};
use reqwest::Proxy;
use std::{str::FromStr, sync::Arc, time::Duration};

pub struct HttpForwardHandler {
    pub(super) client: reqwest::Client,
}

impl HttpForwardHandler {
    pub const NAME: &'static str = "http_forward";

    pub fn new() -> Arc<Self> {
        let mut builder = reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_secs(
                config::get_config().services.forward.connect_timeout,
            ))
            .read_timeout(Duration::from_secs(config::get_config().services.forward.read_timeout))
            .timeout(Duration::from_secs(config::get_config().services.forward.total_timeout))
            .connection_verbose(config::get_config().services.forward.verbose);
        if let Some(proxy) = &config::get_config().services.forward.http_proxy {
            builder = builder.proxy(Proxy::http(proxy).expect("parse http proxy addr error"));
        }
        Arc::new(Self {
            client: builder.build().expect("build http client error"),
        })
    }

    // Extract the upstream URL from the request headers.
    fn get_upstream_url(&self, incoming: Arc<HttpIncomingRequest>) -> Result<String> {
        let upstream_header_name = config::get_config()
            .services
            .forward
            .upstream_destination_header_name
            .to_owned();

        let upstream_base_uri = incoming
            .headers
            .get(&upstream_header_name)
            .map(|h| h.to_owned().unwrap_or_default())
            .ok_or_else(||
                // Only record warning logs instead of error stack
                anyhow::anyhow!(
                    format!("Missing upstream destination header with '{}'", upstream_header_name)
                ))?;

        // If the upstream base URL ends with a slash and the path starts with a slash to prevent duplicate slash.
        let url = if upstream_base_uri.ends_with('/') && incoming.path.starts_with('/') {
            format!("{}{}", upstream_base_uri, &incoming.path[1..])
        } else if !upstream_base_uri.ends_with('/') && !incoming.path.starts_with('/') {
            format!("{}/{}", upstream_base_uri, incoming.path)
        } else {
            format!("{}{}", upstream_base_uri, incoming.path)
        };

        debug!("Extracted the upstream uri: {}", url);
        Ok(url)
    }

    // Forward the request to the upstream server.
    async fn do_forward_request(
        &self,
        incoming: Arc<HttpIncomingRequest>,
        forward_url: String,
    ) -> Result<Response<Body>> {
        let upstream_header = config::get_config()
            .services
            .forward
            .upstream_destination_header_name
            .as_str()
            .to_uppercase();

        info!(
            "Forwarding request to upstream with host: {} path: {}, query: {}",
            incoming.host.to_owned().unwrap_or_default(),
            incoming.path,
            incoming.query.to_owned().unwrap_or_default(),
        );

        let mut req_builder = self
            .client
            .request(Method::from_str(incoming.method.as_str())?, forward_url);

        // Copy original request headers, but exclude certain headers
        for (name, value) in incoming.headers.iter() {
            // Skip certain headers, such as custom upstream destination header and connection related headers.
            let name = name.to_uppercase();
            if name != upstream_header && name != "POST" && name != "CONNECTION" {
                for v in value.iter() {
                    req_builder = req_builder.header(name.to_owned(), v);
                }
            }
        }

        // Addidtional set the request body if provided.
        // The body is type of axum::Bytes is cheaply cloneable and thereby shareable unlimited amount.
        if let Some(body) = incoming.body.to_owned() {
            req_builder = req_builder.body(body);
        }

        // Execute the request.
        let resp = req_builder.send().await?;

        let status = resp.status();
        let headers = resp.headers().clone();
        let bytes = resp
            .bytes()
            .await
            .context("Failed to read response body from upstream")?;

        info!(
            "Forwarded response from upstream status: {}, host: {} path: {}, query: {}, headers: {:?}",
            status,
            incoming.host.to_owned().unwrap_or_default(),
            incoming.path,
            incoming.query.to_owned().unwrap_or_default(),
            headers
        );

        // Build the response.
        let mut response = Response::builder()
            .status(status.as_u16())
            .body(Body::from(bytes))
            .context("Failed to build response")?;

        // Copy the headers from the upstream response.
        let resp_headers = response.headers_mut();
        for (name, value) in headers {
            if let Some(name) = name {
                if name != header::CONNECTION {
                    resp_headers.insert(name, value);
                }
            }
        }

        Ok(response)
    }
}

#[async_trait]
impl IForwarder for HttpForwardHandler {
    async fn init(&self) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn http_forward(&self, incoming: Arc<HttpIncomingRequest>) -> Result<Response<Body>> {
        match self.get_upstream_url(incoming.to_owned()) {
            Ok(url) => self.do_forward_request(incoming.to_owned(), url).await,
            Err(err) => Err(err),
        }
    }
}
