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

use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use axum::{
    body::{to_bytes, Body},
    http::{
        self,
        uri::{Port, Scheme},
        HeaderValue,
    },
    response::Response,
};
use hyper::{header, HeaderMap};
use reqwest::Proxy;

use crate::config::config;

use super::forward_handler::IForwardHandler;

pub struct HttpForwardHandler {
    pub(super) client: reqwest::Client,
}

impl HttpForwardHandler {
    pub fn new() -> Self {
        let mut builder = reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_secs(config::CFG.botwaf.forward.connect_timeout))
            .read_timeout(Duration::from_secs(config::CFG.botwaf.forward.read_timeout))
            .timeout(Duration::from_secs(config::CFG.botwaf.forward.total_timeout))
            .connection_verbose(config::CFG.botwaf.forward.verbose);
        if let Some(proxy) = &config::CFG.botwaf.forward.http_proxy {
            builder = builder.proxy(Proxy::http(proxy).expect("parse http proxy addr error"));
        }
        Self {
            client: builder.build().expect("build http client error"),
        }
    }

    // Extract the upstream URL from the request headers.
    fn get_upstream_url(&self, headers: &HeaderMap, path: &str) -> Result<String> {
        let upstream_header_name = config::CFG.botwaf.forward.upstream_destination_header_name.to_owned();
        let upstream_base_uri = headers.get(&upstream_header_name).and_then(|h| h.to_str().ok()).ok_or_else(||
                // Only record warning logs instead of error stack
                anyhow::anyhow!(
                    format!("Missing upstream destination header with '{}'", upstream_header_name)
                ))?;

        // If the upstream base URL ends with a slash and the path starts with a slash to prevent duplicate slash.
        let url = if upstream_base_uri.ends_with('/') && path.starts_with('/') {
            format!("{}{}", upstream_base_uri, &path[1..])
        } else if !upstream_base_uri.ends_with('/') && !path.starts_with('/') {
            format!("{}/{}", upstream_base_uri, path)
        } else {
            format!("{}{}", upstream_base_uri, path)
        };

        tracing::debug!("Extracted the upstream uri: {}", url);
        Ok(url)
    }

    // Forward the request to the upstream server.
    async fn do_forward_request(
        &self,
        method: reqwest::Method,
        url: String,
        headers: HeaderMap,
        body: Option<Body>,
    ) -> Result<Response<Body>> {
        let upstream_header = config::CFG.botwaf.forward.upstream_destination_header_name.as_str();

        tracing::info!("Forwarding request to upstream: {}", url.to_owned());

        let mut req_builder = self.client.request(method, url.to_owned());

        // Copy original request headers, but exclude certain headers
        for (name, value) in headers.iter() {
            // Skip certain headers, such as custom upstream destination header and connection related headers.
            if name != upstream_header && name != header::HOST && name != header::CONNECTION {
                req_builder = req_builder.header(name, value);
            }
        }

        // Addidtional set the request body if provided.
        if let Some(body) = body {
            let bytes = to_bytes(body, usize::MAX).await.context("Failed to read request body")?;
            req_builder = req_builder.body(bytes);
        }

        // Execute the request.
        let resp = req_builder.send().await?;

        let status = resp.status();
        let headers = resp.headers().clone();
        let bytes = resp.bytes().await.context("Failed to read response body from upstream")?;

        tracing::info!(
            "Forwarded response from upstream status: {}, url: {}, headers: {:?}",
            status,
            url,
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
impl IForwardHandler for HttpForwardHandler {
    #[allow(unused)]
    async fn http_forward(
        &self,
        method: &http::Method,
        scheme: Option<&Scheme>,
        host: Option<&str>,
        port: Option<Port<&str>>,
        path: &str,
        headers: &HeaderMap<HeaderValue>,
        body: Option<Body>,
    ) -> Result<Response<Body>> {
        match self.get_upstream_url(headers, path) {
            Ok(url) => self.do_forward_request(method.to_owned(), url, headers.to_owned(), body).await,
            Err(err) => Err(err),
        }
    }
}
