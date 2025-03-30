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
    config::config::AppConfig,
    sys::{handler::auth_handler::PrincipalType, route::auth_router::EXCLUDED_PREFIX_PATHS},
};
use axum::body::Body;
use botwaf_types::auth::{LoggedResponse, TokenWrapper};
use botwaf_utils::{base64s::Base64Helper, webs};
use chrono::{Duration, Utc};
use hyper::{HeaderMap, Response, StatusCode, Uri};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tower_cookies::cookie::Cookie;

lazy_static! {
    // singleton instance.
    static ref SECURITY_CONTEXT: Arc<SecurityContext> = Arc::new(SecurityContext::new());
}

pub static DEFAULT_BY: &'static str = "0";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthUserClaims {
    pub ptype: PrincipalType,
    pub uid: i64,
    pub uname: String,
    pub email: String,
    pub exp: usize,
    pub ext: Option<HashMap<String, String>>,
}

pub fn create_jwt(
    config: &Arc<AppConfig>,
    ptype: &PrincipalType,
    uid: i64,
    uname: &str,
    email: &str,
    is_refresh: bool,
    extra_claims: Option<HashMap<String, String>>,
) -> String {
    let expiration = Utc::now()
        .checked_add_signed(Duration::milliseconds(if is_refresh {
            config.auth.jwt_validity_rk.unwrap() as i64
        } else {
            config.auth.jwt_validity_ak.unwrap() as i64
        }))
        .expect("valid timestamp")
        .timestamp();

    let claims = AuthUserClaims {
        ptype: ptype.to_owned(),
        uid: uid.to_owned(),
        uname: uname.to_owned(),
        email: email.to_owned(),
        exp: expiration as usize,
        ext: extra_claims,
    };

    let secret = &Base64Helper::decode(&config.auth_jwt_secret.to_owned()).unwrap();
    let header = Header::new(config.auth_jwt_algorithm);
    encode(&header, &claims, &EncodingKey::from_secret(secret)).expect("Failed to encode jwt")
}

pub fn validate_jwt(config: &Arc<AppConfig>, token: &str) -> Result<AuthUserClaims, jsonwebtoken::errors::Error> {
    let secret = &Base64Helper::decode(&config.auth_jwt_secret.to_owned()).unwrap();
    let validation = Validation::new(config.auth_jwt_algorithm);
    let token_data = decode::<AuthUserClaims>(token, &DecodingKey::from_secret(secret), &validation)?;
    Ok(token_data.claims)
}

pub fn auth_resp_redirect_or_json(
    config: &Arc<AppConfig>,
    headers: &HeaderMap,
    redirect_url: &str,
    status: StatusCode,
    message: &str,
    cookies: Option<(Option<Cookie>, Option<Cookie>, Option<Cookie>)>,
) -> Response<Body> {
    let (ak, rk, _) = match &cookies {
        Some(triple) => (
            triple.to_owned().0.map(|c| TokenWrapper {
                value: c.value().to_string(),
                expires_in: config.auth.jwt_validity_ak.unwrap(),
            }),
            triple.to_owned().1.map(|c| TokenWrapper {
                value: c.value().to_string(),
                expires_in: config.auth.jwt_validity_rk.unwrap(),
            }),
            triple.2.to_owned(),
        ),
        None => (None, None, None),
    };

    let json = LoggedResponse {
        errcode: status.as_u16() as i16,
        errmsg: message.to_string(),
        access_token: ak,
        refresh_token: rk,
        redirect_url: Some(join_context_path(&config, redirect_url.to_owned())),
    };
    let json_str = serde_json::to_string(&json).unwrap();

    webs::response_redirect_or_json(
        status,
        headers,
        cookies,
        &json.redirect_url.unwrap(),
        &message,
        &json_str,
    )
}

// Time-constant safety message comparison.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0, |acc, (x, y)| acc | (x ^ y)) == 0
}

pub fn clean_context_path<'a>(ctx_path: &'a Option<String>, path: &'a str) -> &'a str {
    match &ctx_path {
        // Remove the prefix context path.
        Some(cp) if cp == "/" => path,
        Some(cp) => path.strip_prefix(cp.as_str()).unwrap_or(&path),
        None => path,
    }
}

pub fn join_context_path(config: &AppConfig, path: String) -> String {
    // Absolute URI not needs to join context path.
    let schema = url::Url::parse(path.as_str())
        .map(|uri| uri.scheme().to_lowercase())
        .unwrap_or_default();
    if schema.starts_with("http") {
        return path;
    }
    match &config.server.context_path {
        // Add the prefix context path.
        Some(cp) if cp == "/" => path,
        Some(cp) => format!("{}{}", cp, path),
        None => path,
    }
}

pub fn is_passed_request(config: &Arc<AppConfig>, uri: &Uri) -> bool {
    let path = clean_context_path(&config.server.context_path, uri.path());
    // 1. Exclude paths that don't require authentication.
    // 1.1 Paths that must be excluded according to the authentication mechanism's requirements.
    // The root path is also excluded by default.
    // each match with start.
    if EXCLUDED_PREFIX_PATHS.iter().any(|p| path.starts_with(p)) {
        return true;
    }
    // 1.2 According to the configuration of anonymous authentication path.
    if config
        .auth_anonymous_glob_matcher
        .as_ref()
        .map(|glob| glob.is_match(path))
        .unwrap_or(false)
    {
        // If it is an anonymous path, pass it directly.
        return true;
    }
    false
}

#[derive(Clone, Debug)]
pub struct SecurityContext {
    pub current_user: Arc<RwLock<Option<AuthUserClaims>>>,
}

impl SecurityContext {
    pub fn new() -> Self {
        SecurityContext {
            current_user: Arc::new(RwLock::new(None)),
        }
    }

    pub fn get_instance() -> Arc<SecurityContext> {
        SECURITY_CONTEXT.clone()
    }

    pub async fn bind(&self, user: Option<AuthUserClaims>) {
        tracing::debug!("Binding from user: {:?}", user);
        match user {
            Some(user) => {
                // Notice: 必须在此函数中执行 write() 获取写锁, 若在外部 routes/auths.rs#auth_middleware() 中获取写锁,
                // 则当在 routes/users.rs#handle_query_users() 中获取读锁时会产生死锁, 因为 RwLock 的释放机制是超出作用域自动释放,
                // 在 auth_middleware() 中写锁的生命周期包含了 handle_query_users() 即没有释放.
                let mut current_user = self.current_user.write().await;
                *current_user = Some(user);
            }
            None => {}
        }
        tracing::debug!("Binded from user: {:?}", self.get().await);
    }

    pub async fn get(&self) -> Option<AuthUserClaims> {
        match self.current_user.try_read() {
            Ok(read_guard) => read_guard.clone(),
            Err(e) => {
                tracing::error!("Unable to acquire read lock. reason: {:?}", e);
                None
            }
        }
    }

    pub async fn get_current_uid(&self) -> Option<i64> {
        match self.get().await {
            Some(claims) => Some(claims.uid),
            None => {
                tracing::error!("No found current user claims sub.");
                None
            }
        }
    }

    pub async fn get_current_uname(&self) -> Option<String> {
        match self.get().await {
            Some(claims) => Some(claims.uname),
            None => {
                tracing::error!("No found current user claims uname.");
                None
            }
        }
    }

    pub async fn get_current_email(&self) -> Option<String> {
        match self.get().await {
            Some(claims) => Some(claims.email),
            None => {
                tracing::error!("No found current user claims email.");
                None
            }
        }
    }

    pub async fn get_current_uname_for_store(&self) -> Option<String> {
        return SecurityContext::get_instance()
            .get_current_email()
            .await
            .or(SecurityContext::get_instance().get_current_uname().await)
            .or(Some(DEFAULT_BY.to_string()));
    }

    pub async fn clear(&self) {
        let mut write_guard = self.current_user.write().await;
        *write_guard = None;
    }
}
