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

use axum::Router;
use botwaf_server::{
    config::{
        config,
        config::{AppConfig, GIT_BUILD_DATE, GIT_COMMIT_HASH, GIT_VERSION},
        swagger,
    },
    context::state::BotwafState,
    mgmt::{apm, health::init as health_router},
    sys::route::{
        auth_router::{auth_middleware, init as auth_router},
        user_router::init as user_router,
    },
};
use botwaf_utils::{panics::PanicHelper, tokio_signal::tokio_graceful_shutdown_signal};
use clap::Command;
use std::{env, sync::Arc};
use tokio::{net::TcpListener, sync::oneshot};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::cmd::management::ManagementServer;

pub struct WebServer {}

impl WebServer {
    pub const COMMAND_NAME: &'static str = "server";

    pub fn build() -> Command {
        Command::new(Self::COMMAND_NAME).about("Run Botwaf ModSec proxy Web Server.")
    }

    #[allow(unused)]
    #[tokio::main]
    pub async fn run(matches: &clap::ArgMatches, verbose: bool) -> () {
        PanicHelper::set_hook_default();

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
        let app_state = BotwafState::new(&config).await;
        tracing::info!("Register Web server middlewares ...");

        // 1. Merge the biz modules routes.
        let expose_router = Router::new().merge(auth_router()).merge(user_router());

        // 2. Merge of all routes.
        let mut app_router = match &config.server.context_path {
            Some(cp) => {
                Router::new()
                    .merge(health_router())
                    .nest(&cp, expose_router) // support the context-path.
                    .with_state(app_state.clone()) // TODO: remove clone
            }
            None => {
                Router::new()
                    .merge(health_router())
                    .merge(expose_router)
                    .with_state(app_state.clone()) // TODO: remove clone
            }
        };

        // 3. Merge the swagger router.
        if config.swagger.enabled {
            app_router = app_router.merge(swagger::init(&config));
        }

        // 4. Finally add the (auth) middlewares.
        // Notice: The settings of middlewares are in order, which will affect the priority of route matching.
        // The later the higher the priority? For example, if auth_middleware is set at the end, it will
        // enter when requesting '/', otherwise it will not enter if it is set at the front, and will
        // directly enter handle_root().
        app_router = app_router.layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn_with_state(app_state, auth_middleware))
                // Optional: add logs to tracing.
                .layer(
                    TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
                        tracing::info_span!(
                            "http_request",
                            method = %request.method(),
                            uri = %request.uri(),
                        )
                    }),
                ),
        );
        //.route_layer(axum::Extension(app_state));

        let bind_addr = config.server.host.clone() + ":" + &config.server.port.to_string();
        tracing::info!("Starting web server on {}", bind_addr);
        let listener = match TcpListener::bind(&bind_addr).await {
            Ok(l) => {
                tracing::info!("Web server is ready on {}", bind_addr);
                l
            }
            Err(e) => {
                tracing::error!("Failed to bind to {}: {}", bind_addr, e);
                panic!("Failed to bind to {}: {}", bind_addr, e);
            }
        };

        match axum::serve(listener, app_router.into_make_service())
            .with_graceful_shutdown(tokio_graceful_shutdown_signal())
            // .tcp_nodelay(true)
            .await
        {
            Ok(_) => {
                tracing::info!("Web server shut down gracefully");
            }
            Err(e) => {
                tracing::error!("Error running web server: {}", e);
                panic!("Error starting API server: {}", e);
            }
        }
    }

    fn print_banner(config: Arc<AppConfig>, verbose: bool) {
        // http://www.network-science.de/ascii/#larry3d,graffiti,basic,drpepper,rounded,roman
        let ascii_name = r#"
     ____                                           
    /\  _`\                                         
    \ \,\L\_\     __   _ __   __  __     __   _ __  
     \/_\__ \   /'__`\/\`'__\/\ \/\ \  /'__`\/\`'__\
       /\ \L\ \/\  __/\ \ \/ \ \ \_/ |/\  __/\ \ \/ 
       \ `\____\ \____\\ \_\  \ \___/ \ \____\\ \_\ 
        \/_____/\/____/ \/_/   \/__/   \/____/ \/_/  (Botwaf)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_no_args() {
        let app = WebServer::build();
        let matches = app.try_get_matches_from(vec![""]).unwrap();
        assert!(matches.subcommand_name().is_none());
    }

    #[test]
    fn test_cli_start_command() {
        let app = WebServer::build();
        let matches = app.try_get_matches_from(vec!["", "start"]).unwrap();
        assert_eq!(matches.subcommand_name(), Some("start"));
    }

    #[test]
    fn test_cli_start_with_config() {
        let app = WebServer::build();
        let matches = app
            .try_get_matches_from(vec!["", "start", "--config", "config.yaml"])
            .unwrap();
        let start_matches = matches.subcommand_matches("start").unwrap();
        assert_eq!(start_matches.get_one::<String>("config").unwrap(), "config.yaml");
    }

    #[test]
    fn test_cli_invalid_command() {
        let app = WebServer::build();
        let result = app.try_get_matches_from(vec!["", "invalid"]);
        assert!(result.is_err());
    }
}
