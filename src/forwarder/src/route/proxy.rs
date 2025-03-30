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

use crate::handler::{
    forwarder::ForwarderManager, forwarder_http::HttpForwardHandler, ipfilter::IPFilterManager,
    ipfilter_redis::RedisIPFilter,
};
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use botwaf_server::config::config;
use botwaf_server::{config::constant::EXCLUDED_PATHS, context::state::BotwafState};
use botwaf_types::forwarder::HttpIncomingRequest;
use regex::Regex;

pub async fn botwaf_middleware(State(state): State<BotwafState>, req: Request<Body>, next: Next) -> Response {
    let uri = req.uri();
    // Skip the excluded paths.
    if EXCLUDED_PATHS.contains(&uri.path()) {
        tracing::debug!("Passing excluded path: {}", &uri.path());
        return next.run(req).await;
    }

    // Wrap to unified incoming request.
    let incoming = HttpIncomingRequest::new(req, config::get_config().services.forward.max_body_bytes).await;

    // Obtain the available IP filter instance.
    let ipfilter = IPFilterManager::get_implementation(RedisIPFilter::NAME.to_owned()).expect(&format!(
        "Failed to get IP filter implementation with {}.",
        RedisIPFilter::NAME.to_owned()
    ));

    // Check if the request client IP address is blocked.
    if ipfilter.is_blocked(incoming.to_owned()).await.unwrap_or(false) {
        let code = StatusCode::from_u16(config::get_config().services.blocked_status_code.unwrap()).unwrap();
        return Response::builder()
            .status(code)
            .body("Access denied by Botwaf IP Filter".into())
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
                .unwrap_or_else(|| "Access denied by Botwaf".to_string());
            tracing::info!("[Botwaf] [AccessDeined] - {}, reason: {}", incoming.path, logmsg);

            // Getting forbidded by modsec rule id.
            let mut rule_id = String::from("Masked");
            if config::get_config().services.allow_addition_modsec_info {
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
            let code = match config::get_config().services.blocked_status_code {
                Some(code) => StatusCode::from_u16(code).unwrap(),
                None => status_code,
            };

            return Response::builder()
                .status(code)
                .header(config::get_config().services.blocked_header_name.to_owned(), rule_id)
                .body("Access denied by Botwaf Threaten".into())
                .unwrap();
        }
    }

    // Forwarding request to the upstream servers.
    let forwarder = ForwarderManager::get_implementation(HttpForwardHandler::NAME.to_owned()).expect(&format!(
        "Failed to get forwarder implementation with {}.",
        HttpForwardHandler::NAME.to_owned()
    ));
    match forwarder.http_forward(incoming.to_owned()).await {
        std::result::Result::Ok(response) => {
            tracing::info!("[Botwaf] [Forwarded] - {}", &incoming.path);
            response
        }
        Err(err) => {
            tracing::warn!("[Botwaf] [ForwardErr] - {} - {}", &incoming.path, err);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Gateway Forwarded Error")).into_response()
        }
    }
}
