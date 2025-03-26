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
use std::time::Duration;

use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_otlp::{new_exporter, ExportConfig, Protocol};
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::Config;
use opentelemetry_sdk::trace::Tracer;
use opentelemetry_sdk::Resource;

use crate::config::config::AppConfig;

pub async fn create_otel_tracer(config: &Arc<AppConfig>) -> Option<Tracer> {
    let mut tracer = None;

    if config.mgmt.enabled && config.mgmt.otel.enabled {
        let _tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(new_exporter().tonic().with_export_config(ExportConfig {
                endpoint: config.mgmt.otel.endpoint.to_string(),
                protocol: match config.mgmt.otel.protocol.to_lowercase().as_str() {
                    "http/protobuf" => Protocol::HttpBinary,
                    "grpc" => Protocol::Grpc,
                    "http/json" => Protocol::HttpJson,
                    _ => Protocol::HttpBinary,
                },
                timeout: Duration::from_millis(config.mgmt.otel.timeout.unwrap()),
            }))
            .with_trace_config(
                // Notice: More OTEL custom configuration use to environment: OTEL_SPAN_xxx, see to: opentelemetry_sdk::trace::config::default()
                Config::default().with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    config.service_name.to_string(),
                )])),
            )
            .install_batch(Tokio)
            .expect("Failed to install OpenTelemetry tracer.");

        // Get a tracer from the provider
        tracer = Some(_tracer);
        //tracer = Some(TracerProvider::tracer(&tracer_provider, "default_tracer"));

        // Make sure all trace data is refreshed.
        global::shutdown_tracer_provider();
    }

    tracer
}
