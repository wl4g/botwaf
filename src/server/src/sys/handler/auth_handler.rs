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

use super::user_handler::{IUserHandler, UserHandler};
use crate::util::auths;
use crate::{config::config::AppConfig, context::state::BotwafState};
use anyhow::{anyhow, Error, Ok};
use async_trait::async_trait;
use botwaf_types::{
    sys::auth::{EthersWalletLoginRequest, GithubUserInfo, LogoutRequest, PasswordLoginRequest, PasswordPubKeyRequest},
    sys::user::{SaveUserRequest, User},
};
use botwaf_utils::rsa_ciphers::RSACipher;
use chrono::Utc;
use common_telemetry::info;
use ethers::types::{Address, Signature};
use hyper::{header, StatusCode};
use lazy_static::lazy_static;
use openidconnect::{core::CoreUserInfoClaims, LanguageTag};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tower_cookies::cookie::{time::Duration, CookieBuilder, SameSite};

pub const AUTH_NONCE_PREFIX: &'static str = "auth:nonce:";
pub const LOGIN_PRIVATE_KEY_PREFIX: &'static str = "login:privatekey:";
pub const LOGOUT_BLACKLIST_PREFIX: &'static str = "logout:blacklist:";

lazy_static! {
    pub static ref LANG_CLAIMS_NAME_KEY: LanguageTag = LanguageTag::new("name".to_owned());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrincipalType {
    Password,
    OIDC,
    Github,
    EtherWallet,
}

#[async_trait]
pub trait IAuthHandler: Send {
    async fn handle_password_pubkey(&self, param: PasswordPubKeyRequest) -> Result<String, Error>;

    async fn handle_password_verify(&self, param: PasswordLoginRequest) -> Result<Arc<User>, Error>;

    async fn handle_auth_create_nonce(&self, sid: &str, nonce: String) -> Result<(), Error>;

    async fn handle_auth_get_nonce(&self, sid: &str) -> Result<Option<String>, Error>;

    async fn handle_auth_callback_oidc(&self, userinfo: CoreUserInfoClaims) -> Result<i64, Error>;

    async fn handle_auth_callback_github(&self, userinfo: GithubUserInfo) -> Result<i64, Error>;

    async fn handle_wallet_verify_ethers(&self, param: EthersWalletLoginRequest) -> Result<i64, Error>;

    async fn handle_login_success(
        &self,
        config: &Arc<AppConfig>,
        ptype: PrincipalType,
        uid: i64,
        uname: &str,
        email: &str,
        headers: &header::HeaderMap,
    ) -> hyper::Response<axum::body::Body>;

    async fn handle_logout(&self, param: LogoutRequest) -> Result<(), Error>;

    fn build_auth_nonce_key(&self, nonce: &str) -> String;

    fn build_login_private_key(&self, fingerprint_token: &str) -> String;

    fn build_logout_blacklist_key(&self, access_token: &str) -> String;
}

pub struct AuthHandler<'a> {
    state: &'a BotwafState,
}

impl<'a> AuthHandler<'a> {
    pub fn new(state: &'a BotwafState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl<'a> IAuthHandler for AuthHandler<'a> {
    async fn handle_password_pubkey(&self, param: PasswordPubKeyRequest) -> Result<String, Error> {
        let pair = RSACipher::new(2048).unwrap();
        // Storage private key to cache.
        let cache = self.state.string_cache.get(&self.state.config);
        let key = self.build_login_private_key(&param.fingerprint_token);
        let value = pair.get_base64_private_key().unwrap();
        match cache.set(key, value, Some(30_000)).await {
            std::result::Result::Ok(_) => {
                info!("Got login pubkey for: {:?}", param);
                Ok(pair.get_base64_public_key().unwrap())
            }
            Err(e) => {
                tracing::error!("Failed to got login pubkey. {:?}, cause: {}", param, e);
                Err(e)
            }
        }
    }

    async fn handle_password_verify(&self, param: PasswordLoginRequest) -> Result<Arc<User>, Error> {
        let cache = self.state.string_cache.get(&self.state.config);
        let key = self.build_login_private_key(&param.fingerprint_token);

        // Getting private key from cache.
        match cache.get(key).await {
            std::result::Result::Ok(value) => {
                match value {
                    Some(base64_private_key) => {
                        tracing::debug!("Got login private key for: {:?}", param);
                        let pair = RSACipher::from_base64(&base64_private_key).unwrap();
                        let hashed_password: Vec<u8> = match pair.decrypt_from_base64(&param.password) {
                            std::result::Result::Ok(p) => p,
                            Err(e) => {
                                return Err(anyhow!(format!("Unable decryption password. {:?}", e.to_string())));
                            }
                        };

                        // Getting user from database.
                        let handler = UserHandler::new(self.state);
                        match handler.get(None, None, None, None, None, None, None, None).await {
                            std::result::Result::Ok(user) => match user {
                                Some(user) => {
                                    let store_hashed_password = user.password.clone().unwrap_or_default().into_bytes();
                                    if auths::constant_time_eq(&hashed_password, &store_hashed_password) {
                                        tracing::debug!("Login success for: {:?}", param);
                                        Ok(user)
                                    } else {
                                        tracing::error!("Login failed for: {:?}", param);
                                        Err(anyhow!("Invalid password"))
                                    }
                                }
                                None => {
                                    let errmsg = format!(
                                        "No login user, Please confirm that the login account is correct. {:?}",
                                        param
                                    );
                                    tracing::error!(errmsg);
                                    Err(anyhow!(errmsg))
                                }
                            },
                            Err(e) => {
                                tracing::error!("Failed to get user. {:?}, cause: {}", param, e);
                                Err(e.into())
                            }
                        }
                    }
                    None => {
                        let errmsg = format!(
                            "No login private key, The operation takes too long? Please refresh and log in again. {:?}",
                            param
                        );
                        tracing::error!(errmsg);
                        Err(anyhow!(errmsg))
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to get login private key. {:?}, cause: {}", param, e);
                Err(e)
            }
        }
    }

    async fn handle_auth_create_nonce(&self, sid: &str, nonce: String) -> Result<(), Error> {
        let cache = self.state.string_cache.get(&self.state.config);

        let key = self.build_logout_blacklist_key(sid);
        let value = nonce;

        // TODO: using expires config? To ensure safety, expire as soon as possible. 10s
        match cache.set(key, value, Some(10_000)).await {
            std::result::Result::Ok(_) => {
                info!("Created auth nonce for {}", sid);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Created auth nonce failed for {}, cause: {}", sid, e);
                Err(e)
            }
        }
    }

    async fn handle_auth_get_nonce(&self, sid: &str) -> Result<Option<String>, Error> {
        let cache = self.state.string_cache.get(&self.state.config);

        let key = self.build_logout_blacklist_key(sid);

        match cache.get(key).await {
            std::result::Result::Ok(nonce) => {
                info!("Got auth nonce for {}", sid);
                Ok(nonce)
            }
            Err(e) => {
                tracing::error!("Get auth nonce failed for {}, cause: {}", sid, e);
                Err(e)
            }
        }
    }

    async fn handle_auth_callback_oidc(&self, userinfo: CoreUserInfoClaims) -> Result<i64, Error> {
        let oidc_sub = userinfo.subject().as_str();
        // let oidc_uname = userinfo.name().map(|n| n.get(Some(&LANG_CLAIMS_NAME_KEY)).map(|u| u.to_string()).unwrap_or_default());
        let oidc_preferred_name = userinfo.preferred_username().map(|c| c.to_string());
        let oidc_email = userinfo.email().map(|c| c.to_string());

        let handler = UserHandler::new(self.state);

        // 1. Get user by oidc uid
        let user = handler
            .get(None, None, None, None, Some(oidc_sub.to_string()), None, None, None)
            .await
            .unwrap();

        // 2. If user exists, update user github subject ID.
        let save_param;
        if user.is_some() {
            save_param = SaveUserRequest {
                id: user.unwrap().base.id,
                name: oidc_preferred_name.to_owned(),
                email: None,
                phone: None,
                password: None,
                oidc_claims_sub: Some(oidc_sub.to_string()),
                oidc_claims_name: oidc_preferred_name,
                oidc_claims_email: oidc_email,
                github_claims_sub: None,
                github_claims_name: None,
                github_claims_email: None,
                google_claims_sub: None,
                google_claims_name: None,
                google_claims_email: None,
                ethers_address: None,
                lang: None,
            };
        } else {
            // 3. If user not exists, create user by github login, which auto register user.
            save_param = SaveUserRequest {
                id: None,
                name: oidc_preferred_name.to_owned(),
                email: None,
                phone: None,
                password: None,
                oidc_claims_sub: Some(oidc_sub.to_string()),
                oidc_claims_name: oidc_preferred_name,
                oidc_claims_email: oidc_email,
                github_claims_sub: None,
                github_claims_name: None,
                github_claims_email: None,
                google_claims_sub: None,
                google_claims_name: None,
                google_claims_email: None,
                ethers_address: None,
                lang: None,
            };
        }

        match handler.save(save_param).await {
            std::result::Result::Ok(uid) => Ok(uid),
            Err(e) => Err(e),
        }
    }

    async fn handle_auth_callback_github(&self, userinfo: GithubUserInfo) -> Result<i64, Error> {
        let github_sub = userinfo.id.expect("github uid is None");
        let github_uname = userinfo.login.expect("github uname is None");
        let github_email = userinfo.email;

        let handler = UserHandler::new(self.state);

        // 1. Get user by github_uid
        let user = handler
            .get(None, None, None, None, None, Some(github_sub.to_string()), None, None)
            .await
            .unwrap();

        // 2. If user exists, update user github subject ID.
        let save_param;
        if user.is_some() {
            save_param = SaveUserRequest {
                id: user.unwrap().base.id,
                name: Some(github_uname.to_string()),
                email: None,
                phone: None,
                password: None,
                oidc_claims_sub: None,
                oidc_claims_name: None,
                oidc_claims_email: None,
                github_claims_sub: Some(github_sub.to_string()),
                github_claims_name: Some(github_uname.to_string()),
                github_claims_email: github_email,
                google_claims_sub: None,
                google_claims_name: None,
                google_claims_email: None,
                ethers_address: None,
                lang: None,
            };
        } else {
            // 3. If user not exists, create user by github login, which auto register user.
            save_param = SaveUserRequest {
                id: None,
                name: Some(github_uname.to_string()),
                email: None,
                phone: None,
                password: None,
                oidc_claims_sub: None,
                oidc_claims_name: None,
                oidc_claims_email: None,
                github_claims_sub: Some(github_sub.to_string()),
                github_claims_name: Some(github_uname.to_string()),
                github_claims_email: github_email,
                google_claims_sub: None,
                google_claims_name: None,
                google_claims_email: None,
                ethers_address: None,
                lang: None,
            };
        }

        match handler.save(save_param).await {
            std::result::Result::Ok(uid) => Ok(uid),
            Err(e) => Err(e),
        }
    }

    async fn handle_wallet_verify_ethers(&self, param: EthersWalletLoginRequest) -> Result<i64, Error> {
        // 1. Convert to Address, Signature.
        let address = Address::from_str(&param.address).map_err(|_| anyhow!("Invalid address"))?;
        let signature = Signature::from_str(&param.signature).map_err(|_| anyhow!("Invalid signature"))?;

        // 2. Verify the signature.
        let result = signature.recover(param.message).map_err(|_| StatusCode::UNAUTHORIZED);
        match result {
            std::result::Result::Ok(recovered_address) => {
                if recovered_address.eq(&address) {
                    let uname = param.address;

                    let handler = UserHandler::new(self.state);
                    let user = handler
                        .get(None, None, None, None, None, None, None, Some(uname.to_owned()))
                        .await
                        .unwrap();

                    // 3. If user exists, update user github subject ID.
                    let save_param;
                    if user.is_some() {
                        save_param = SaveUserRequest {
                            id: user.unwrap().base.id,
                            name: Some(uname.to_owned()),
                            email: None,
                            phone: None,
                            password: None,
                            oidc_claims_sub: None,
                            oidc_claims_name: None,
                            oidc_claims_email: None,
                            github_claims_sub: None,
                            github_claims_name: None,
                            github_claims_email: None,
                            google_claims_sub: None,
                            google_claims_name: None,
                            google_claims_email: None,
                            ethers_address: Some(uname),
                            lang: None,
                        };
                    } else {
                        // 4. If user not exists, create user by github login, which auto register user.
                        save_param = SaveUserRequest {
                            id: None,
                            name: Some(uname.to_owned()),
                            email: None,
                            phone: None,
                            password: None,
                            oidc_claims_sub: None,
                            oidc_claims_name: None,
                            oidc_claims_email: None,
                            github_claims_sub: None,
                            github_claims_name: None,
                            github_claims_email: None,
                            google_claims_sub: None,
                            google_claims_name: None,
                            google_claims_email: None,
                            ethers_address: Some(uname),
                            lang: None,
                        };
                    }

                    // 5. save user info
                    match handler.save(save_param).await {
                        std::result::Result::Ok(uid) => Ok(uid),
                        Err(e) => Err(e),
                    }
                } else {
                    tracing::error!("Failed to verify wallet signature.");
                    Err(anyhow!(StatusCode::UNAUTHORIZED))
                }
            }
            Err(e) => {
                tracing::error!("Failed to verify wallet signature. cause: {}", e);
                Err(anyhow!(e))
            }
        }
    }

    async fn handle_login_success(
        &self,
        config: &Arc<AppConfig>,
        ptype: PrincipalType,
        uid: i64,
        uname: &str,
        email: &str,
        headers: &header::HeaderMap,
    ) -> hyper::Response<axum::body::Body> {
        // TODO: 附加更多自定义 JWT 信息
        let extra_claims = HashMap::new();
        let ak = auths::create_jwt(config, &ptype, uid, uname, email, false, Some(extra_claims));
        let rk = auths::create_jwt(config, &ptype, uid, uname, email, true, None);

        let ak_cookie = CookieBuilder::new(&config.auth_jwt_ak_name, ak)
            .path("/")
            .max_age(Duration::milliseconds(config.auth.jwt_validity_ak.unwrap() as i64))
            //.secure(true) // true: indicates that only https requests will carry
            .http_only(true)
            .same_site(SameSite::Strict)
            .build();

        let rk_cookie = CookieBuilder::new(&config.auth_jwt_rk_name, rk)
            .path("/")
            .max_age(Duration::milliseconds(config.auth.jwt_validity_rk.unwrap() as i64))
            //.secure(true) // true: indicates that only https requests will carry
            .http_only(true)
            .same_site(SameSite::Strict)
            .build();

        auths::auth_resp_redirect_or_json(
            &config,
            headers,
            config.auth.success_url.to_owned().unwrap().as_str(),
            StatusCode::OK,
            "Authenticated",
            Some((Some(ak_cookie), Some(rk_cookie), None)),
        )
    }

    async fn handle_logout(&self, param: LogoutRequest) -> Result<(), Error> {
        let cache = self.state.string_cache.get(&self.state.config);

        // Add current jwt token to cache blacklist, expiration time is less than now time - id_token issue time.
        let ak = match param.access_token {
            Some(v) => v.to_string(),
            None => {
                return Err(Error::msg("access_token is None"));
            }
        };
        let key = self.build_logout_blacklist_key(ak.as_str());
        let value = Utc::now().timestamp_millis().to_string();
        match cache.set(key, value, Some(3600_000)).await {
            std::result::Result::Ok(_) => {
                info!("Logout success for {}", ak);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Logout failed: {}, cause: {}", ak, e);
                Err(e)
            }
        }
    }

    fn build_auth_nonce_key(&self, nonce: &str) -> String {
        format!("{}:{}", AUTH_NONCE_PREFIX, nonce)
    }

    fn build_login_private_key(&self, fingerprint_token: &str) -> String {
        format!("{}:{}", LOGIN_PRIVATE_KEY_PREFIX, fingerprint_token)
    }

    fn build_logout_blacklist_key(&self, access_token: &str) -> String {
        format!("{}:{}", LOGOUT_BLACKLIST_PREFIX, access_token)
    }
}
