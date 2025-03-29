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

use axum::{routing::get, Router};
use axum_prometheus::PrometheusMetricLayer;
use botwaf_server::{config::config::AppConfig, mgmt};
use tokio::{sync::oneshot, task::JoinHandle};

pub struct ManagementServer {}

impl ManagementServer {
    #[allow(unused)]
    pub async fn start(config: &Arc<AppConfig>, verbose: bool, signal_sender: oneshot::Sender<()>) -> JoinHandle<()> {
        let (prometheus_layer, _) = PrometheusMetricLayer::pair();

        let app: Router = Router::new()
            .route("/metrics", get(mgmt::apm::metrics::handle_metrics))
            .layer(prometheus_layer);

        let bind_addr = config.mgmt.host.clone() + ":" + &config.mgmt.port.to_string();
        tracing::info!("Starting Management server on {}", bind_addr);

        tokio::spawn(async move {
            // When started call to signal sender.
            let _ = signal_sender.send(());
            axum::serve(
                tokio::net::TcpListener::bind(&bind_addr).await.unwrap(),
                app.into_make_service(),
            )
            .await
            .unwrap_or_else(|e| panic!("Error starting management server: {}", e));
        })
    }
}
