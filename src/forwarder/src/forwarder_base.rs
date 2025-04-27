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
    forwarder_http::HttpForwardHandler,
    ipfilter::{ipfilter::IPFilterManager, ipfilter_redis::RedisIPFilter},
};
use anyhow::{Error, Result};
use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use botwaf_server::{config::config, context::state::BotwafState, util::auths};
use botwaf_types::modules::forward::forwarder::HttpIncomingRequest;
use hyper::StatusCode;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[async_trait]
pub trait IForwarder {
    async fn init(&self) -> Result<()>;
    async fn http_forward(&self, incoming: Arc<HttpIncomingRequest>) -> Result<Response<Body>>;
}

lazy_static! {
    /// RwLock Notices:
    /// Read lock (shared lock):
    /// Allow multiple threads to hold read locks at the same time.
    /// Read locks will not block each other, and all read lock holders can read data concurrently.
    /// Write lock (exclusive lock):
    /// Only one thread can hold a write lock.
    /// The write lock will block all other threads' read-write lock requests until the write lock is released.
    /// Mutual exclusion rules for read-write locks:
    /// When a thread holds a write lock, other threads cannot obtain read or write locks. ---Therefore, the phantom read problem of the database is avoided
    /// When one or more threads hold a read lock, other threads cannot obtain a write lock. ---Therefore, RwLock is only suitable for scenarios with more reads and less writes, such as cache systems, configuration file reading, etc.
    static ref SINGLE_INSTANCE: RwLock<BotwafForwarderManager> = RwLock::new(BotwafForwarderManager::new());
}

pub struct BotwafForwarderManager {
    pub implementations: HashMap<String, Arc<dyn IForwarder + Send + Sync>>,
}

impl BotwafForwarderManager {
    fn new() -> Self {
        BotwafForwarderManager {
            implementations: HashMap::new(),
        }
    }

    pub fn get() -> &'static RwLock<BotwafForwarderManager> {
        &SINGLE_INSTANCE
    }

    pub async fn init() {
        IPFilterManager::init().await;

        tracing::info!("Register Botwaf Http IForwarder ...");
        match Self::get()
            .write() // If acquire fails, then it block until acquired.
            .unwrap() // If acquire fails, then it should panic.
            .register(HttpForwardHandler::NAME.to_owned(), HttpForwardHandler::new())
        {
            Ok(registered) => {
                tracing::info!("Initializing Botwaf Http IForwarder ...");
                let _ = registered.init().await;
            }
            Err(e) => panic!("Failed to register Http IForwarder: {}", e),
        }
    }

    fn register<T: IForwarder + Send + Sync + 'static>(
        &mut self,
        name: String,
        handler: Arc<T>,
    ) -> Result<Arc<T>, Error> {
        if self.implementations.contains_key(&name) {
            tracing::debug!("Already register the Forwarder '{}'", name);
            return Ok(handler);
        }
        self.implementations.insert(name, handler.to_owned());
        Ok(handler)
    }

    pub fn get_implementation(name: String) -> Result<Arc<dyn IForwarder + Send + Sync>, Error> {
        // If the read lock is poisoned, the program will panic.
        let this = BotwafForwarderManager::get().read().unwrap();
        if let Some(implementation) = this.implementations.get(&name) {
            Ok(implementation.to_owned())
        } else {
            let errmsg = format!("Could not obtain registered Forwarder '{}'.", name);
            return Err(Error::msg(errmsg));
        }
    }

    pub async fn botwaf_middleware(State(state): State<BotwafState>, req: Request<Body>, next: Next) -> Response {
        let uri = req.uri();

        // 1. Exclude if there is any path excluded.
        if auths::is_anonymous_request(&state.config, uri) {
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
        let forwarder =
            BotwafForwarderManager::get_implementation(HttpForwardHandler::NAME.to_owned()).expect(&format!(
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
}
