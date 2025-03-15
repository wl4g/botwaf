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

use anyhow::Result;
use async_trait::async_trait;
use axum::{
    body::Body,
    http::{
        self,
        uri::{Port, Scheme},
        HeaderValue,
    },
    response::Response,
};
use hyper::HeaderMap;

#[async_trait]
pub trait IForwardHandler {
    async fn http_forward(
        &self,
        method: &http::Method,
        scheme: Option<&Scheme>,
        host: Option<&str>,
        port: Option<Port<&str>>,
        path: &str,
        headers: &HeaderMap<HeaderValue>,
        body: Option<Body>,
    ) -> Result<Response<Body>>;
}
