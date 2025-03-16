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
    cache::redis::StringRedisCache,
    config::config::{self, GIT_BUILD_DATE, GIT_COMMIT_HASH, GIT_VERSION},
    logging,
    server::{
        forward_handler::IForwardHandler, forward_handler_http::HttpForwardHandler, ipfilter_handler::IPFilterHandler,
        ipfilter_handler_redis::RedisIPFilterHandler,
    },
    types::server::HttpIncomingRequest,
};
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
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;

// Default router URIs paths to excluding.
pub const URI_STATIC: &str = "/static/*file";
pub const URI_HEALTHZ: &str = "/healthz";
pub const EXCLUDED_PATHS: [&str; 2] = [URI_STATIC, URI_HEALTHZ];

#[derive(Clone)]
struct BotWafState {
    modsec_instance: Arc<ModSecurity>,
    modsec_rules: Arc<Rules>,
    #[allow(unused)]
    redis_cache: Arc<StringRedisCache>,
    ipfilter_handler: Arc<dyn IPFilterHandler + Send + Sync>,
    forward_handler: Arc<dyn IForwardHandler + Send + Sync>,
}

impl BotWafState {
    fn new() -> Self {
        let ms = Arc::new(ModSecurity::default());
        let mut rules = Rules::new();
        for rule in config::CFG.botwaf.static_rules.clone() {
            if rule.kind == "RAW" {
                tracing::info!("Loading the security static rule: {} - {} - {}", rule.name, rule.kind, rule.value);
                rules.add_plain(rule.value.as_str()).expect("Failed to add rules");
            }
        }
        let rules = Arc::new(rules);
        let redis_cache = Arc::new(StringRedisCache::new(&config::CFG.cache.redis));
        BotWafState {
            modsec_instance: ms,
            modsec_rules: rules,
            redis_cache: redis_cache.to_owned(),
            ipfilter_handler: RedisIPFilterHandler::new(redis_cache, config::CFG.botwaf.blocked_header_name.clone()),
            forward_handler: Arc::new(HttpForwardHandler::new()),
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
    if state.ipfilter_handler.is_blocked(incoming.to_owned()).await.unwrap_or(false) {
        let code = StatusCode::from_u16(config::CFG.botwaf.blocked_status_code.unwrap()).unwrap();
        return Response::builder().status(code).body("Access denied by BotWaf IP Filter".into()).unwrap();
    }

    // Create a ModSecurity engine transaction with rules.
    let mut transaction = state
        .modsec_instance
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
    transaction.process_request_headers().expect("Error processing request headers");
    // Process the request body with ModSecurity engine.
    let req_body = incoming.body.to_owned().unwrap_or_default().to_vec();
    transaction.append_request_body(&req_body).expect("Error processing request body");

    // Check if the request is blocked by ModSecurity engine.
    if let Some(intervention) = transaction.intervention() {
        if intervention.status() == 401 || intervention.status() == 403 {
            let status_code = StatusCode::from_u16(intervention.status() as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
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
    match state.forward_handler.http_forward(incoming.to_owned()).await {
        Ok(response) => {
            tracing::info!("[BotWaf] [Forwarded] - {}", &incoming.path);
            response
        }
        Err(err) => {
            tracing::warn!("[BotWaf] [ForwardErr] - {} - {}", &incoming.path, err);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Gateway Forwarded Error")).into_response()
        }
    }
}

pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    // http://www.network-science.de/ascii/#larry3d,graffiti,basic,drpepper,rounded,roman
    let ascii_name = r#"
    ____            __    __      __               ___      ____                                     
    /\  _`\         /\ \__/\ \  __/\ \            /'___\    /\  _`\                                   
    \ \ \L\ \    ___\ \ ,_\ \ \/\ \ \ \     __   /\ \__/    \ \,\L\_\     __   _ __   __  __     __   
     \ \  _ <'  / __`\ \ \/\ \ \ \ \ \ \  /'__`\ \ \ ,__\    \/_\__ \   /'__`\/\`'__\/\ \/\ \  /'__`\ 
      \ \ \L\ \/\ \L\ \ \ \_\ \ \_/ \_\ \/\ \L\.\_\ \ \_/      /\ \L\ \/\  __/\ \ \/ \ \ \_/ |/\  __/ 
       \ \____/\ \____/\ \__\\ `\___x___/\ \__/.\_\\ \_\       \ `\____\ \____\\ \_\  \ \___/ \ \____\
        \/___/  \/___/  \/__/ '\/__//__/  \/__/\/_/ \/_/        \/_____/\/____/ \/_/   \/__/   \/____/
"#;
    eprintln!("");
    eprintln!("{}", ascii_name);
    eprintln!("                Program Version: {:?}", GIT_VERSION);
    eprintln!("                Package Version: {:?}", env!("CARGO_PKG_VERSION").to_string());
    eprintln!("                Git Commit Hash: {:?}", GIT_COMMIT_HASH);
    eprintln!("                 Git Build Date: {:?}", GIT_BUILD_DATE);

    logging::init_components().await;

    let botwaf_state = BotWafState::new();

    let app_routes = Router::new()
        .route(URI_HEALTHZ, axum::routing::get(|| async { "BotWaf is Running!" }))
        .layer(ServiceBuilder::new().layer(axum::middleware::from_fn_with_state(botwaf_state, botwaf_middleware)));

    let bind_addr = config::CFG.server.host.clone() + ":" + &config::CFG.server.port.to_string();
    tracing::info!("Starting Botwaf web server on {}", bind_addr);
    let listener = match TcpListener::bind(&bind_addr).await {
        Ok(l) => {
            tracing::info!("Botwaf Web server is ready on {}", bind_addr);
            l
        }
        Err(e) => {
            tracing::error!("Failed to bind to {}: {}", bind_addr, e);
            panic!("Failed to bind to {}: {}", bind_addr, e);
        }
    };
    match axum::serve(listener, app_routes.into_make_service()).await {
        Ok(_) => {
            tracing::info!("Botwaf Web server shut down gracefully");
        }
        Err(e) => {
            tracing::error!("Error running web server: {}", e);
            panic!("Error starting API server: {}", e);
        }
    }

    Ok(())
}
