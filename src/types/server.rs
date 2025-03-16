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

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    body::{to_bytes, Body, Bytes},
    extract::Request,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HttpThreatSampleRecord {
    content: String,
    label: String,
    metadata: serde_json::Value,
}

#[derive(Clone)]
pub struct HttpIncomingRequest {
    pub method: String,
    pub scheme: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub headers: HashMap<String, Option<String>>,
    pub path: String,
    pub query: Option<String>,
    pub body: Option<Bytes>,
    pub client_ip: Option<String>,
}

impl HttpIncomingRequest {
    pub async fn new(req: Request<Body>) -> Arc<Self> {
        let (parts, body) = req.into_parts();
        // TODO limit body size by configuration.
        let bytes = to_bytes(body, 65535).await.expect("Failed to collect request body");
        let (req, body) = (Request::from_parts(parts, Body::from(bytes.clone())), bytes);
        let uri = req.uri();

        // Extract request headers.
        let headers = req
            .headers()
            .iter()
            .map(|(name, value)| {
                let key = name.as_str().to_string();
                let value = value.to_str().map(|v| v.to_string()).ok();
                (key, value)
            })
            .collect();

        // Extract axum request client IP by using the X-Forwarded-For or X-Real-IP or the request remote address.
        let client_ip = req
            .headers()
            .get("X-Forwarded-For")
            .or(req.headers().get("X-Real-IP"))
            .map(|addr| addr.to_str().map(|s| s.to_string()).unwrap_or_default())
            .or_else(|| req.extensions().get::<SocketAddr>().map(|addr| addr.ip().to_string()));

        Arc::new(HttpIncomingRequest {
            method: req.method().to_string(),
            scheme: uri.scheme().map(|s| s.to_string()),
            host: uri.host().map(|s| s.to_string()),
            port: uri.port_u16(),
            headers,
            path: uri.path().to_string(),
            query: uri.query().map(|s| s.to_string()),
            //body: Some(String::from_utf8_lossy(&body).to_string()),
            body: Some(body),
            client_ip,
        })
    }
}
