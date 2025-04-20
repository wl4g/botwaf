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

use crate::config::config::{AppDBType, CacheProvider};
use crate::context::state::BotwafState;
use async_trait::async_trait;
use axum::{extract::State, response::IntoResponse, routing::get, Router};
use botwaf_types::{sys::user::User, PageRequest};
use hyper::StatusCode;
use serde::Serialize;
use std::collections::HashMap;

pub(crate) const HEALTHZ_URI: &str = "/_/healthz";
// TODO Addidtional more health checkers.
// pub(crate) const STARTUP_HEALTHZ_URI: &str = "/_/healthz/startup";
// pub(crate) const READNESS_HEALTHZ_URI: &str = "/_/healthz/readness";
// pub(crate) const LIVENESS_HEALTHZ_URI: &str = "/_/healthz/liveness";

#[async_trait]
pub(crate) trait HealthChecker: Send + Sync {
    async fn check(&self, state: &BotwafState) -> HealthCheckResult;
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct HealthCheckResult {
    pub status: String,
    pub details: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct SQLiteChecker {}

impl SQLiteChecker {
    pub fn new() -> Self {
        SQLiteChecker {}
    }

    async fn is_sqlite_connected(&self, state: &BotwafState) -> bool {
        match &state.config.appdb.db_type {
            AppDBType::SQLITE => {
                let repo = state.user_repo.lock().await;
                match repo
                    .get(&state.config)
                    .select(User::default(), PageRequest::default())
                    .await
                {
                    Ok(_) => true,
                    Err(e) => {
                        tracing::error!("SQLite connection check failed: {}", e);
                        false
                    }
                }
            }
            _ => true, // If not enabled, it is considered healthy.
        }
    }
}

#[async_trait]
impl HealthChecker for SQLiteChecker {
    async fn check(&self, state: &BotwafState) -> HealthCheckResult {
        let status = if self.is_sqlite_connected(state).await {
            "UP"
        } else {
            "DOWN"
        };
        HealthCheckResult {
            status: "sqlite".to_string(),
            details: HashMap::from([("sqlite".to_string(), status.to_string())]),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MongoChecker {}

impl MongoChecker {
    pub fn new() -> Self {
        MongoChecker {}
    }

    async fn is_mongo_connected(&self, state: &BotwafState) -> bool {
        match &state.config.appdb.db_type {
            AppDBType::MONGODB => {
                let repo = state.user_repo.lock().await;
                match repo
                    .get(&state.config)
                    .select(User::default(), PageRequest::default())
                    .await
                {
                    Ok(_) => true,
                    Err(e) => {
                        tracing::error!("Mongo connection check failed: {}", e);
                        false
                    }
                }
            }
            _ => true, // If not enabled, it is considered healthy.
        }
    }
}

#[async_trait]
impl HealthChecker for MongoChecker {
    async fn check(&self, state: &BotwafState) -> HealthCheckResult {
        let status = if self.is_mongo_connected(state).await {
            "UP"
        } else {
            "DOWN"
        };
        HealthCheckResult {
            status: "mongo".to_string(),
            details: HashMap::from([("mongo".to_string(), status.to_string())]),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RedisClusterChecker {}

impl RedisClusterChecker {
    pub fn new() -> Self {
        RedisClusterChecker {}
    }

    async fn is_redis_cluster_connected(&self, state: &BotwafState) -> bool {
        match &state.config.cache.provider {
            CacheProvider::REDIS => {
                let cache = state.string_cache.get(&state.config);
                match cache.get("".to_string()).await {
                    Ok(_) => true,
                    Err(e) => {
                        tracing::error!("Redis cluster connection check failed: {}", e);
                        false
                    }
                }
            }
            _ => true, // If not enabled, it is considered healthy.
        }
    }
}

#[async_trait]
impl HealthChecker for RedisClusterChecker {
    async fn check(&self, state: &BotwafState) -> HealthCheckResult {
        let status = if self.is_redis_cluster_connected(state).await {
            "UP"
        } else {
            "DOWN"
        };
        HealthCheckResult {
            status: "redis-cluster".to_string(),
            details: HashMap::from([("redis-cluster".to_string(), status.to_string())]),
        }
    }
}

pub fn init() -> Router<BotwafState> {
    Router::new().route(HEALTHZ_URI, get(handle_healthz))
    // .route(STARTUP_HEALTHZ_URI, get(handle_healthz_startup))
    // .route(READNESS_HEALTHZ_URI, get(handle_healthz_readness))
    // .route(READNESS_HEALTHZ_URI, get(handle_healthz_liveness))
}

async fn handle_healthz(State(state): State<BotwafState>) -> impl IntoResponse {
    let mut result = HealthCheckResult {
        status: "UP".to_string(),
        details: HashMap::new(),
    };

    let sqlite_check = SQLiteChecker::new().check(&state).await;
    result.details.extend(sqlite_check.details);
    if sqlite_check.status == "DOWN" {
        result.status = "DOWN".to_string();
    }

    let mongo_check = MongoChecker::new().check(&state).await;
    result.details.extend(mongo_check.details);
    if mongo_check.status == "DOWN" {
        result.status = "DOWN".to_string();
    }

    let redis_cluster_check = RedisClusterChecker::new().check(&state).await;
    result.details.extend(redis_cluster_check.details);
    if redis_cluster_check.status == "DOWN" {
        result.status = "DOWN".to_string();
    }

    (StatusCode::OK, serde_json::to_string(&result).unwrap())
}
