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

use anyhow::{Error, Ok};
use axum::Router;
use tokio::net::TcpListener;

use crate::{
    botwaf_shutdown_signal,
    config::{
        config::{self, GIT_BUILD_DATE, GIT_COMMIT_HASH, GIT_VERSION},
        constant::URI_HEALTHZ,
    },
    logging,
    server::server::BotWafState,
    updater::updater_base::BotwafUpdaterManager,
    verifier::verifier_base::BotwafVerifierManager,
};

pub async fn start() -> Result<(), Error> {
    // http://www.network-science.de/ascii/#larry3d,graffiti,basic,drpepper,rounded,roman
    let ascii_name = r#"
____    __                        __            ___                             
/\  _`\ /\ \__                    /\ \          /\_ \                            
\ \,\L\_\ \ ,_\    __      ___    \_\ \     __  \//\ \     ___     ___      __   
 \/_\__ \\ \ \/  /'__`\  /' _ `\  /'_` \  /'__`\  \ \ \   / __`\ /' _ `\  /'__`\ 
   /\ \L\ \ \ \_/\ \L\.\_/\ \/\ \/\ \L\ \/\ \L\.\_ \_\ \_/\ \L\ \/\ \/\ \/\  __/ 
   \ `\____\ \__\ \__/.\_\ \_\ \_\ \___,_\ \__/.\_\/\____\ \____/\ \_\ \_\ \____\
    \/_____/\/__/\/__/\/_/\/_/\/_/\/__,_ /\/__/\/_/\/____/\/___/  \/_/\/_/\/____/  (Botwaf)
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

    logging::init_components().await;

    BotwafUpdaterManager::init().await;
    BotwafVerifierManager::init().await;

    let botwaf_state = BotWafState::new().await;
    let app_router = build_app_router(botwaf_state).await?;

    let bind_addr = config::CFG.server.host.clone() + ":" + &config::CFG.server.port.to_string();
    tracing::info!("Starting Botwaf Standalone server on {}", bind_addr);

    let listener = match TcpListener::bind(&bind_addr).await {
        std::result::Result::Ok(l) => {
            tracing::info!("Botwaf Standalone server is ready on {}", bind_addr);
            l
        }
        Err(e) => {
            tracing::error!("Failed to bind to {}: {}", bind_addr, e);
            panic!("Failed to bind to {}: {}", bind_addr, e);
        }
    };

    match axum::serve(listener, app_router.into_make_service())
        .with_graceful_shutdown(botwaf_shutdown_signal())
        .await
    {
        std::result::Result::Ok(_) => {
            tracing::info!("Botwaf Standalone Server shut down gracefully");
        }
        Err(e) => {
            tracing::error!("Error running Standalone server: {}", e);
            panic!("Error starting Standalone Server: {}", e);
        }
    }

    Ok(())
}

pub async fn build_app_router(_: BotWafState) -> Result<Router, Error> {
    let app_router = Router::new().route(
        URI_HEALTHZ,
        axum::routing::get(|| async { "BotWaf Standalone Server is Running!" }),
    );

    Ok(app_router)
}
