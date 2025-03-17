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

use crate::{
    botwaf_shutdown_signal,
    cache::redis::StringRedisCache,
    config::{
        config::{self, GIT_BUILD_DATE, GIT_COMMIT_HASH, GIT_VERSION},
        constant::{EXCLUDED_PATHS, URI_HEALTHZ},
    },
    handler::{llm_handler::ILLMHandler, llm_handler_langchain::LangchainLLMHandler},
    logging,
    server::{
        forwarder::IForwarder, forwarder_http::HttpForwardHandler, ipfilter::IPFilter,
        ipfilter_redis::RedisIPFilterHandler,
    },
    types::server::HttpIncomingRequest,
};
use anyhow::{Error, Ok};
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Router,
};
use modsecurity::{ModSecurity, Rules};
use regex::Regex;
use std::{env, sync::Arc};
use tokio::{net::TcpListener, signal};
use tower::ServiceBuilder;

#[derive(Clone)]
pub struct BotWafState {
    pub modsec_engine: Arc<ModSecurity>,
    pub modsec_rules: Arc<Rules>,
    #[allow(unused)]
    pub redis_cache: Arc<StringRedisCache>,
    pub ipfilter: Arc<dyn IPFilter + Send + Sync>,
    pub forwarder: Arc<dyn IForwarder + Send + Sync>,
    pub llm_handler: Arc<dyn ILLMHandler + Send + Sync>,
}

impl BotWafState {
    pub async fn new() -> Self {
        let modsec_engine = Arc::new(ModSecurity::default());

        let mut rules = Rules::new();
        for rule in config::CFG.botwaf.static_rules.clone() {
            if rule.kind == "RAW" {
                tracing::info!(
                    "Loading the security static rule: {} - {} - {}",
                    rule.name,
                    rule.kind,
                    rule.value
                );
                rules.add_plain(rule.value.as_str()).expect("Failed to add rules");
            }
        }
        let modsec_rules = Arc::new(rules);

        let redis_cache = Arc::new(StringRedisCache::new(&config::CFG.cache.redis));

        BotWafState {
            modsec_engine,
            modsec_rules,
            redis_cache: redis_cache.to_owned(),
            ipfilter: RedisIPFilterHandler::new(redis_cache, config::CFG.botwaf.blocked_header_name.clone()),
            forwarder: Arc::new(HttpForwardHandler::new()),
            llm_handler: Arc::new(LangchainLLMHandler::new().await),
        }
    }
}

async fn botwaf_middleware(State(state): State<BotWafState>, req: Request<Body>, next: Next) -> impl IntoResponse {
    let uri = req.uri();
    // Skip the excluded paths.
    if EXCLUDED_PATHS.contains(&uri.path()) {
        tracing::debug!("Passing excluded path: {}", &uri.path());
        return next.run(req).await;
    }

    // Wrap to unified incoming request.
    let incoming = HttpIncomingRequest::new(req).await;

    // Check if the request client IP address is blocked.
    if state.ipfilter.is_blocked(incoming.to_owned()).await.unwrap_or(false) {
        let code = StatusCode::from_u16(config::CFG.botwaf.blocked_status_code.unwrap()).unwrap();
        return Response::builder()
            .status(code)
            .body("Access denied by BotWaf IP Filter".into())
            .unwrap();
    }

    // Create a ModSecurity engine transaction with rules.
    let mut transaction = state
        .modsec_engine
        .transaction_builder()
        .with_rules(&state.modsec_rules)
        .build()
        .expect("Error building transaction");

    // Process the request URI with ModSecurity engine.
    transaction
        .process_uri(&incoming.path, &incoming.method, "1.1")
        .expect("Error processing URI");
    // Process the request headers with ModSecurity engine.
    for (key, value) in incoming.headers.iter() {
        transaction
            .add_request_header(key, value.as_ref().unwrap_or(&"".to_string()))
            .expect("Error add request header.");
    }
    transaction
        .process_request_headers()
        .expect("Error processing request headers");
    // Process the request body with ModSecurity engine.
    let req_body = incoming.body.to_owned().unwrap_or_default().to_vec();
    transaction
        .append_request_body(&req_body)
        .expect("Error processing request body");

    // Check if the request is blocked by ModSecurity engine.
    if let Some(intervention) = transaction.intervention() {
        if intervention.status() == 401 || intervention.status() == 403 {
            let status_code =
                StatusCode::from_u16(intervention.status() as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            let logmsg = intervention
                .log()
                .map(|msg| msg.to_string())
                .unwrap_or_else(|| "Access denied by BotWaf".to_string());
            tracing::info!("[BotWaf] [AccessDeined] - {}, reason: {}", incoming.path, logmsg);

            // Getting forbidded by modsec rule id.
            let mut rule_id = String::from("Masked");
            if config::CFG.botwaf.allow_addition_modsec_info {
                let re = Regex::new(r#"\[id "\s*(\d+)\s*"\]"#).unwrap();
                rule_id = intervention
                    .log()
                    .and_then(|log| re.captures(log))
                    .and_then(|caps| caps.get(1))
                    .map(|m| m.as_str())
                    .unwrap_or("Unknown")
                    .to_owned();
            }

            // Determining ModSec rejected response status code.
            let code = match config::CFG.botwaf.blocked_status_code {
                Some(code) => StatusCode::from_u16(code).unwrap(),
                None => status_code,
            };

            return Response::builder()
                .status(code)
                .header(config::CFG.botwaf.blocked_header_name.to_owned(), rule_id)
                .body("Access denied by BotWaf Threaten".into())
                .unwrap();
        }
    }

    // Forwarding request to the upstream servers.
    match state.forwarder.http_forward(incoming.to_owned()).await {
        std::result::Result::Ok(response) => {
            tracing::info!("[BotWaf] [Forwarded] - {}", &incoming.path);
            response
        }
        Err(err) => {
            tracing::warn!("[BotWaf] [ForwardErr] - {} - {}", &incoming.path, err);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Gateway Forwarded Error")).into_response()
        }
    }
}

pub async fn start() -> Result<(), Error> {
    // http://www.network-science.de/ascii/#larry3d,graffiti,basic,drpepper,rounded,roman
    let ascii_name = r#"
____                                     
/\  _`\                                   
\ \,\L\_\     __   _ __   __  __     __   
 \/_\__ \   /'__`\/\`'__\/\ \/\ \  /'__`\ 
   /\ \L\ \/\  __/\ \ \/ \ \ \_/ |/\  __/ 
   \ `\____\ \____\\ \_\  \ \___/ \ \____\
    \/_____/\/____/ \/_/   \/__/   \/____/  (Botwaf)
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

    let botwaf_state = BotWafState::new().await;
    let app_router = build_app_router(botwaf_state).await?;

    let bind_addr = config::CFG.server.host.clone() + ":" + &config::CFG.server.port.to_string();
    tracing::info!("Starting Botwaf web server on {}", bind_addr);

    let listener = match TcpListener::bind(&bind_addr).await {
        std::result::Result::Ok(l) => {
            tracing::info!("Botwaf Web server is ready on {}", bind_addr);
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
            tracing::info!("Botwaf Web server shut down gracefully");
        }
        Err(e) => {
            tracing::error!("Error running web server: {}", e);
            panic!("Error starting API server: {}", e);
        }
    }

    Ok(())
}

pub async fn build_app_router(state: BotWafState) -> Result<Router, Error> {
    let app_router = Router::new()
        .route(
            URI_HEALTHZ,
            axum::routing::get(|| async { "BotWaf Web Server is Running!" }),
        )
        .layer(ServiceBuilder::new().layer(axum::middleware::from_fn_with_state(state, botwaf_middleware)));

    Ok(app_router)
}
