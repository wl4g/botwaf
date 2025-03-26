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

use std::sync::Arc;

use lazy_static::lazy_static;
use prometheus::{ Registry, Counter, Histogram, Encoder, TextEncoder };

use crate::config::config::AppConfig;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    pub static ref MY_HTTP_REQUESTS_TOTAL: Counter = Counter::new(
        "my_http_requests_total",
        "My Total number of HTTP requests"
    ).expect("My metric can be created");

    pub static ref MY_HTTP_REQUEST_DURATION: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "http_request_duration_seconds",
            "My HTTP request duration in seconds"
        )
    ).expect("My metric can be created");
    // Register more metrics...
}

#[allow(unused)]
pub async fn handle_metrics() -> String {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&REGISTRY.gather(), &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

#[allow(unused)]
pub async fn init_metrics(config: &Arc<AppConfig>) {
    if config.mgmt.enabled {
        tracing::info!("Custom metrics starting ...");
        REGISTRY.register(Box::new(MY_HTTP_REQUESTS_TOTAL.clone())).expect(
            "collector can be registered"
        );
        REGISTRY.register(Box::new(MY_HTTP_REQUEST_DURATION.clone())).expect(
            "collector can be registered"
        );
        // Register more metrics...
    }
}
