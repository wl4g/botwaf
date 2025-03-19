use std::env;

use crate::{
    botwaf_shutdown_signal,
    config::{
        config::{self, GIT_BUILD_DATE, GIT_COMMIT_HASH, GIT_VERSION},
        constant::URI_HEALTHZ,
    },
    logging,
    server::server::BotWafState,
    updater::updater_base::BotwafUpdaterManager,
};
use anyhow::Error;
use axum::Router;
use tokio::net::TcpListener;

pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    // http://www.network-science.de/ascii/#larry3d,graffiti,basic,drpepper,rounded,roman
    let ascii_name = r#"
__  __              __            __                   
/\ \/\ \            /\ \          /\ \__                
\ \ \ \ \  _____    \_\ \     __  \ \ ,_\    __   _ __  
 \ \ \ \ \/\ '__`\  /'_` \  /'__`\ \ \ \/  /'__`\/\`'__\
  \ \ \_\ \ \ \L\ \/\ \L\ \/\ \L\.\_\ \ \_/\  __/\ \ \/ 
   \ \_____\ \ ,__/\ \___,_\ \__/.\_\\ \__\ \____\\ \_\ 
    \/_____/\ \ \/  \/__,_ /\/__/\/_/ \/__/\/____/ \/_/ 
             \ \_\                                      
              \/_/                                        (Botwaf)
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

    let botwaf_state = BotWafState::new().await;
    let app_router = build_app_router(botwaf_state).await?;

    let bind_addr = config::CFG.server.host.clone() + ":" + &config::CFG.server.port.to_string();
    tracing::info!("Starting Botwaf Updater server on {}", bind_addr);

    let listener = match TcpListener::bind(&bind_addr).await {
        std::result::Result::Ok(l) => {
            tracing::info!("Botwaf Updater server is ready on {}", bind_addr);
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
            tracing::info!("Botwaf Updater Server shut down gracefully");
        }
        Err(e) => {
            tracing::error!("Error running Updater server: {}", e);
            panic!("Error starting Updater Server: {}", e);
        }
    }

    Ok(())
}

pub async fn build_app_router(_: BotWafState) -> Result<Router, Error> {
    let app_router = Router::new().route(
        URI_HEALTHZ,
        axum::routing::get(|| async { "BotWaf Updater Server is Running!" }),
    );

    Ok(app_router)
}
