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

use crate::cmd::management::ManagementServer;
use axum::Router;
use botwaf_server::config::config::AppConfig;
use botwaf_server::context::state::BotwafState;
use botwaf_server::{
    config::{
        config::{self, GIT_BUILD_DATE, GIT_COMMIT_HASH, GIT_VERSION},
        constant::URI_HEALTHZ,
    },
    mgmt::apm,
};
use botwaf_utils::tokio_signal::tokio_graceful_shutdown_signal;
use botwaf_verifier::verifier_base::BotwafVerifierManager;
use clap::Command;
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

pub struct VerifierServer {}

impl VerifierServer {
    pub const COMMAND_NAME: &'static str = "verifier";

    pub fn build() -> Command {
        Command::new(Self::COMMAND_NAME).about("Run Botwaf AI generated ModSec rules Verifier.")
    }

    #[allow(unused)]
    #[tokio::main]
    pub async fn run(matches: &clap::ArgMatches, verbose: bool) -> () {
        std::panic::set_hook(Box::new(|info| {
            let info = info.to_string().replace('\n', " ");
            tracing::error!(%info);
            eprintln!("Oh, Occur Panic Error ::\n{}", info)
        }));

        let config = config::get_config();

        Self::print_banner(config.to_owned(), verbose);

        // Initial APM components.
        apm::init_components(&config).await;

        let (signal_s, signal_r) = oneshot::channel();
        let signal_handle = ManagementServer::start(&config, true, signal_s).await;

        signal_r.await.expect("Failed to start Management server.");
        tracing::info!("Management server is ready");

        Self::start(&config, true).await;

        signal_handle.await.unwrap();
    }

    #[allow(unused)]
    async fn start(config: &Arc<AppConfig>, verbose: bool) {
        BotwafVerifierManager::init().await;

        let app_state = BotwafState::new(&config).await;

        let bind_addr = config.server.host.clone() + ":" + &config.server.port.to_string();
        tracing::info!("Starting Botwaf Verifier server on {}", bind_addr);
        let listener = match TcpListener::bind(&bind_addr).await {
            Ok(l) => {
                tracing::info!("Botwaf Verifier server is ready on {}", bind_addr);
                l
            }
            Err(e) => {
                tracing::error!("Failed to bind to {}: {}", bind_addr, e);
                panic!("Failed to bind to {}: {}", bind_addr, e);
            }
        };

        let app_router = Router::new().merge(Self::health_router(app_state));
        match axum::serve(listener, app_router.into_make_service())
            .with_graceful_shutdown(tokio_graceful_shutdown_signal())
            // .tcp_nodelay(true)
            .await
        {
            Ok(_) => {
                tracing::info!("Botwaf Verifier server shut down gracefully");
            }
            Err(e) => {
                tracing::error!("Error running web server: {}", e);
                panic!("Error start Botwaf Verifier server: {}", e);
            }
        }
    }

    fn print_banner(config: Arc<AppConfig>, verbose: bool) {
        // http://www.network-science.de/ascii/#larry3d,graffiti,basic,drpepper,rounded,roman
        let ascii_name = r#"
     __  __                        ___                     
    /\ \/\ \                 __  /'___\ __                 
    \ \ \ \ \     __   _ __ /\_\/\ \__//\_\     __   _ __  
     \ \ \ \ \  /'__`\/\`'__\/\ \ \ ,__\/\ \  /'__`\/\`'__\
      \ \ \_/ \/\  __/\ \ \/ \ \ \ \ \_/\ \ \/\  __/\ \ \/ 
       \ `\___/\ \____\\ \_\  \ \_\ \_\  \ \_\ \____\\ \_\ 
        `\/__/  \/____/ \/_/   \/_/\/_/   \/_/\/____/ \/_/  (Botwaf)
 "#;
        eprintln!("");
        eprintln!("{}", ascii_name);
        eprintln!("                Program Version: {:?}", GIT_VERSION);
        eprintln!(
            "                Package Version: {:?}",
            env!("CARGO_PKG_VERSION").to_string()
        );
        eprintln!("                Git Commit Hash: {:?}", GIT_COMMIT_HASH);
        eprintln!("                 Git Build Date: {:?}", GIT_BUILD_DATE);
        let path = env::var("BOTWAF_CFG_PATH").unwrap_or("none".to_string());
        eprintln!("        Configuration file path: {:?}", path);
        eprintln!(
            "            Web Serve listen on: \"{}://{}:{}\"",
            "http", &config.server.host, config.server.port
        );
        if config.mgmt.enabled {
            eprintln!(
                "     Management serve listen on: \"{}://{}:{}\"",
                "http", config.mgmt.host, config.mgmt.port
            );
            if config.mgmt.tokio_console.enabled {
                #[cfg(feature = "tokio-console")]
                let server_addr = &config.mgmt.tokio_console.server_bind;
                #[cfg(feature = "tokio-console")]
                eprintln!("   TokioConsole serve listen on: \"{}://{}\"", "http", server_addr);
            }
            if config.mgmt.pyroscope.enabled {
                #[cfg(feature = "profiling")]
                let server_url = &config.mgmt.pyroscope.server_url;
                #[cfg(feature = "profiling")]
                eprintln!("     Pyroscope agent connect to: \"{}\"", server_url);
            }
            if config.mgmt.otel.enabled {
                let endpoint = &config.mgmt.otel.endpoint;
                eprintln!("          Otel agent connect to: \"{}\"", endpoint);
            }
        }
        if verbose {
            let config_json = serde_json::to_string(&config.inner).unwrap_or_default();
            eprintln!("Configuration loaded: {}", config_json);
        }
        eprintln!("");
    }

    fn health_router(_: BotwafState) -> Router {
        Router::new().route(
            URI_HEALTHZ,
            axum::routing::get(|| async { "Botwaf Verifier Server is Running!" }),
        )
    }
}
