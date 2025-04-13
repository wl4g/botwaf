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

use crate::config::config::AppConfig;
use crate::mgmt::apm::otel::create_otel_tracer;
use std::sync::Arc;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;

pub mod logging;
pub mod metrics;
pub mod otel;

pub async fn init_components(config: &Arc<AppConfig>) {
    // Setup logging+tracing layers.
    let (route_layer, _) = tracing_subscriber::reload::Layer::new(logging::default_log_route_layer());
    let (stderr_layer, _) = tracing_subscriber::reload::Layer::new(logging::default_log_stderr_layer(config));
    let level_layer = logging::default_log_levels_layer();

    let subscriber = tracing_subscriber::registry()
        .with(route_layer)
        .with(stderr_layer)
        .with(level_layer);

    // Create OpenTelemetry layer if tracer is available.
    let otel_layer = create_otel_tracer(config).await.map(OpenTelemetryLayer::new);
    // Add OpenTelemetry layer if available.
    let subscriber = subscriber.with(otel_layer);

    // Setup profiling for tokio-console layers.
    if config.mgmt.enabled && config.mgmt.tokio_console.enabled {
        // Notice: Use optional dependencies to avoid slow auto compilation during debugg, because if rely
        // on console-subscriber, need to enable RUSTFLAGS="--cfg tokio_unstable" which
        // will invalidate the compile-time cache.
        // TODO: removed this feature in the future instead of using a common-telemetry/logging.rs
        #[cfg(feature = "profiling-tokio-console")]
        let server_addr = config
            .mgmt
            .tokio_console
            .server_bind
            .as_str()
            .parse::<std::net::SocketAddr>()
            .expect("Failed to parse server address");
        // TODO: removed this feature in the future instead of using a common-telemetry/logging.rs
        #[cfg(feature = "profiling-tokio-console")]
        let subscriber = subscriber.with(
            console_subscriber::ConsoleLayer::builder()
                .with_default_env()
                .server_addr(server_addr)
                .retention(std::time::Duration::from_secs(config.mgmt.tokio_console.retention))
                .spawn(),
        );
        // set the subscriber as the default for the application
        tracing::subscriber::set_global_default(subscriber).unwrap();
    } else {
        // set the subscriber as the default for the application
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }

    // Setup custom metrics.
    metrics::init_metrics(config).await;

    // Setup profiling.
    // common_telemetry::pyroscope_agent::init_profiling(config).await;
}
