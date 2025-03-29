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

use std::env;

use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::Sampler;
use opentelemetry_semantic_conventions::resource;
use serde::{Deserialize, Serialize};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{filter, EnvFilter, Registry};

use crate::tracing_sampler::{create_sampler, TracingSampleOptions};
pub use crate::{debug, error, info, trace, warn};

const DEFAULT_OTLP_ENDPOINT: &str = "http://localhost:4317";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingOptions {
    pub dir: String,
    pub level: Option<String>,
    pub enable_otlp_tracing: bool,
    pub otlp_endpoint: Option<String>,
    pub tracing_sample_ratio: Option<TracingSampleOptions>,
    pub append_stdout: bool,
}

impl PartialEq for LoggingOptions {
    fn eq(&self, other: &Self) -> bool {
        self.dir == other.dir
            && self.level == other.level
            && self.enable_otlp_tracing == other.enable_otlp_tracing
            && self.otlp_endpoint == other.otlp_endpoint
            && self.tracing_sample_ratio == other.tracing_sample_ratio
            && self.append_stdout == other.append_stdout
    }
}

impl Eq for LoggingOptions {}

impl Default for LoggingOptions {
    fn default() -> Self {
        Self {
            dir: "/tmp/botwaf/logs".to_string(),
            level: None,
            enable_otlp_tracing: false,
            otlp_endpoint: None,
            tracing_sample_ratio: None,
            append_stdout: true,
        }
    }
}

#[derive(Default)]
pub struct TracingOptions {
    #[cfg(feature = "tokio-console")]
    pub tokio_console_addr: Option<String>,
}

const DEFAULT_LOG_TARGETS: &str = "info";

#[allow(clippy::print_stdout)]
pub fn init_global_logging(
    app_name: &str,
    opts: &LoggingOptions,
    tracing_opts: TracingOptions,
    node_id: Option<String>,
) -> Vec<WorkerGuard> {
    let mut guards = vec![];
    let dir = &opts.dir;
    let level = &opts.level;
    let enable_otlp_tracing = opts.enable_otlp_tracing;

    // Enable log compatible layer to convert log record to tracing span.
    LogTracer::init().expect("log tracer must be valid");

    // stdout log layer.
    let stdout_logging_layer = if opts.append_stdout {
        let (stdout_writer, stdout_guard) = tracing_appender::non_blocking(std::io::stdout());
        guards.push(stdout_guard);

        Some(
            Layer::new()
                .with_writer(stdout_writer)
                .with_ansi(atty::is(atty::Stream::Stdout)),
        )
    } else {
        None
    };

    // file log layer.
    let rolling_appender = RollingFileAppender::new(Rotation::HOURLY, dir, app_name);
    let (rolling_writer, rolling_writer_guard) = tracing_appender::non_blocking(rolling_appender);
    let file_logging_layer = Layer::new().with_writer(rolling_writer).with_ansi(false);
    guards.push(rolling_writer_guard);

    // error file log layer.
    let err_rolling_appender = RollingFileAppender::new(Rotation::HOURLY, dir, format!("{}-{}", app_name, "err"));
    let (err_rolling_writer, err_rolling_writer_guard) = tracing_appender::non_blocking(err_rolling_appender);
    let err_file_logging_layer = Layer::new().with_writer(err_rolling_writer).with_ansi(false);
    guards.push(err_rolling_writer_guard);

    // resolve log level settings from:
    // - options from command line or config files
    // - environment variable: RUST_LOG
    // - default settings
    let rust_log_env = std::env::var(EnvFilter::DEFAULT_ENV).ok();
    let targets_string = level
        .as_deref()
        .or(rust_log_env.as_deref())
        .unwrap_or(DEFAULT_LOG_TARGETS);
    let filter = targets_string
        .parse::<filter::Targets>()
        .expect("error parsing log level string");
    let sampler = opts
        .tracing_sample_ratio
        .as_ref()
        .map(create_sampler)
        .map(Sampler::ParentBased)
        .unwrap_or(Sampler::ParentBased(Box::new(Sampler::AlwaysOn)));
    // Must enable 'tokio_unstable' cfg to use this feature.
    // For example: `RUSTFLAGS="--cfg tokio_unstable" cargo run -F common-telemetry/console -- standalone start`
    #[cfg(feature = "tokio-console")]
    let subscriber = {
        let tokio_console_layer = if let Some(tokio_console_addr) = &tracing_opts.tokio_console_addr {
            let addr: std::net::SocketAddr = tokio_console_addr.parse().unwrap_or_else(|e| {
                panic!("Invalid binding address '{tokio_console_addr}' for tokio-console: {e}");
            });
            println!("tokio-console listening on {addr}");

            Some(console_subscriber::ConsoleLayer::builder().server_addr(addr).spawn())
        } else {
            None
        };

        let stdout_logging_layer = stdout_logging_layer.map(|x| x.with_filter(filter.clone()));

        let file_logging_layer = file_logging_layer.with_filter(filter);

        Registry::default()
            .with(tokio_console_layer)
            .with(stdout_logging_layer)
            .with(file_logging_layer)
            .with(err_file_logging_layer.with_filter(filter::LevelFilter::ERROR))
    };

    // consume the `tracing_opts`, to avoid "unused" warnings
    let _ = tracing_opts;

    #[cfg(not(feature = "tokio-console"))]
    let subscriber = Registry::default()
        .with(filter)
        .with(stdout_logging_layer)
        .with(file_logging_layer)
        .with(err_file_logging_layer.with_filter(filter::LevelFilter::ERROR));

    if enable_otlp_tracing {
        global::set_text_map_propagator(TraceContextPropagator::new());
        // otlp exporter
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter().tonic().with_endpoint(
                    opts.otlp_endpoint
                        .as_ref()
                        .map(|e| format!("http://{}", e))
                        .unwrap_or(DEFAULT_OTLP_ENDPOINT.to_string()),
                ),
            )
            .with_trace_config(opentelemetry_sdk::trace::config().with_sampler(sampler).with_resource(
                opentelemetry_sdk::Resource::new(vec![
                    KeyValue::new(resource::SERVICE_NAME, app_name.to_string()),
                    KeyValue::new(resource::SERVICE_INSTANCE_ID, node_id.unwrap_or("none".to_string())),
                    KeyValue::new(resource::SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
                    KeyValue::new(resource::PROCESS_PID, std::process::id().to_string()),
                ]),
            ))
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .expect("otlp tracer install failed");
        let tracing_layer = Some(tracing_opentelemetry::layer().with_tracer(tracer));
        let subscriber = subscriber.with(tracing_layer);
        tracing::subscriber::set_global_default(subscriber).expect("error setting global tracing subscriber");
    } else {
        tracing::subscriber::set_global_default(subscriber).expect("error setting global tracing subscriber");
    }

    guards
}
