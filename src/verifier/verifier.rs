use std::env;

use crate::{
    botwaf_shutdown_signal,
    config::{
        config::{self, GIT_BUILD_DATE, GIT_COMMIT_HASH, GIT_VERSION},
        constant::URI_HEALTHZ,
    },
    logging,
    server::server::BotWafState,
    verifier::verifier_base::BotwafVerifierManager,
};
use anyhow::Error;
use axum::Router;
use tokio::net::TcpListener;

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

    logging::init_components().await;

    BotwafVerifierManager::start().await;

    let botwaf_state = BotWafState::new().await;
    let app_router = build_app_router(botwaf_state).await?;

    let bind_addr = config::CFG.server.host.clone() + ":" + &config::CFG.server.port.to_string();
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
        .with_graceful_shutdown(botwaf_shutdown_signal())
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

pub async fn build_app_router(_: BotWafState) -> Result<Router, Error> {
    let app_router = Router::new().route(
        URI_HEALTHZ,
        axum::routing::get(|| async { "BotWaf Verifier Server is Running!" }),
    );

    Ok(app_router)
}
