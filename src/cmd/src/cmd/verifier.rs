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

use crate::verifier_base::BotwafVerifierManager;
use anyhow::Error;
use axum::Router;
use botwaf_server::config::{
    config::{self, GIT_BUILD_DATE, GIT_COMMIT_HASH, GIT_VERSION},
    constant::URI_HEALTHZ,
};
use botwaf_server::context::state::BotwafState;
use botwaf_utils::tokio_signal::tokio_graceful_shutdown_signal;
use std::env;
use tokio::net::TcpListener;

/// Run Botwaf AI generated ModSec rules Verifier.
pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
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
    eprintln!("                Program Version: {}", GIT_VERSION);
    eprintln!(
        "                Package Version: {}",
        env!("CARGO_PKG_VERSION").to_string()
    );
    eprintln!("                Git Commit Hash: {}", GIT_COMMIT_HASH);
    eprintln!("                 Git Build Date: {}", GIT_BUILD_DATE);
    let load_config = env::var("BOTWAF_CFG_PATH").unwrap_or("Default".to_string());
    eprintln!("             Load Configuration: {}", load_config);

    // TODO::
    //logging::init_components().await;

    BotwafVerifierManager::init().await;

    let botwaf_state = BotwafState::new().await;
    let app_router = build_app_router(botwaf_state).await?;

    let bind_addr = config::get_config().server.host.clone() + ":" + &config::get_config().server.port.to_string();
    tracing::info!("Starting Botwaf Verifier server on {}", bind_addr);

    let listener = match TcpListener::bind(&bind_addr).await {
        std::result::Result::Ok(l) => {
            tracing::info!("Botwaf Verifier server is ready on {}", bind_addr);
            l
        }
        Err(e) => {
            tracing::error!("Failed to bind to {}: {}", bind_addr, e);
            panic!("Failed to bind to {}: {}", bind_addr, e);
        }
    };

    match axum::serve(listener, app_router.into_make_service())
        .with_graceful_shutdown(tokio_graceful_shutdown_signal())
        .await
    {
        std::result::Result::Ok(_) => {
            tracing::info!("Botwaf Verifier Server shut down gracefully");
        }
        Err(e) => {
            tracing::error!("Error running Verifier server: {}", e);
            panic!("Error starting Verifier Server: {}", e);
        }
    }

    Ok(())
}

pub async fn build_app_router(_: BotwafState) -> Result<Router, Error> {
    let app_router = Router::new().route(
        URI_HEALTHZ,
        axum::routing::get(|| async { "Botwaf Verifier Server is Running!" }),
    );

    Ok(app_router)
}
