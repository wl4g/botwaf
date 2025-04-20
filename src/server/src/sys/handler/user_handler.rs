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

use crate::context::state::BotwafState;
use anyhow::{Error, Ok};
use async_trait::async_trait;
use botwaf_types::sys::user::{DeleteUserRequest, QueryUserRequest, SaveUserRequest, SaveUserRequestWith, User};
use botwaf_types::{BaseBean, PageRequest, PageResponse};
use common_audit_log::audit_log;
use std::sync::Arc;

#[async_trait]
pub trait IUserHandler: Send {
    async fn get(
        &self,
        id: Option<i64>,
        name: Option<String>,
        email: Option<String>,
        phone: Option<String>,
        oidc_claims_sub: Option<String>,
        github_claims_sub: Option<String>,
        google_claims_sub: Option<String>,
        ethers_address: Option<String>,
    ) -> Result<Option<Arc<User>>, Error>;

    async fn set(
        &self,
        id: Option<i64>,
        name: Option<String>,
        email: Option<String>,
        phone: Option<String>,
        oidc_claims_sub: Option<String>,
        github_claims_sub: Option<String>,
        google_claims_sub: Option<String>,
        ethers_address: Option<String>,
        param: SaveUserRequestWith,
    ) -> Result<(), Error>;

    async fn find(&self, param: QueryUserRequest, page: PageRequest) -> Result<(PageResponse, Vec<User>), Error>;

    async fn save(&self, param: SaveUserRequest) -> Result<i64, Error>;

    async fn delete(&self, param: DeleteUserRequest) -> Result<u64, Error>;
}

pub struct UserHandler<'a> {
    state: &'a BotwafState,
}

impl<'a> UserHandler<'a> {
    pub fn new(state: &'a BotwafState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl<'a> IUserHandler for UserHandler<'a> {
    async fn get(
        &self,
        id: Option<i64>,
        name: Option<String>,
        email: Option<String>,
        phone: Option<String>,
        oidc_claims_sub: Option<String>,
        github_claims_sub: Option<String>,
        google_claims_sub: Option<String>,
        ethers_address: Option<String>,
    ) -> Result<Option<Arc<User>>, Error> {
        let param = User {
            base: BaseBean::new(id, None, None),
            name,
            email,
            phone,
            password: None,
            oidc_claims_sub,
            oidc_claims_name: None,
            oidc_claims_email: None,
            github_claims_sub,
            github_claims_name: None,
            github_claims_email: None,
            google_claims_sub,
            google_claims_name: None,
            google_claims_email: None,
            ethers_address,
            lang: None,
        };

        let repo = self.state.user_repo.lock().await;
        let res = repo
            .get(&self.state.config)
            .select(param, PageRequest::default())
            .await
            .unwrap()
            .1;

        if res.len() > 0 {
            // Notice: Fuck, I don't know why?
            // 如果这里使用 Rc 包装而不是 Arc，那么在 routes/auths.rs中 fn init() { Router::new().route("/auth/callback/github", get(callback_github)) } 会报错泛型参数匹配错误.
            let user = Arc::new(res.get(0).unwrap().clone());
            return Ok(Some(user));
        } else {
            Ok(None)
        }
    }

    async fn set(
        &self,
        id: Option<i64>,
        name: Option<String>,
        email: Option<String>,
        phone: Option<String>,
        oidc_claims_sub: Option<String>,
        github_claims_sub: Option<String>,
        google_claims_sub: Option<String>,
        ethers_address: Option<String>,
        param: SaveUserRequestWith,
    ) -> Result<(), Error> {
        match self
            .get(
                id,
                name,
                email,
                phone,
                oidc_claims_sub,
                github_claims_sub,
                google_claims_sub,
                ethers_address,
            )
            .await
        {
            std::result::Result::Ok(Some(user)) => {
                let mut save_param = SaveUserRequest {
                    id: None,
                    name: param.name,
                    email: param.email,
                    phone: param.phone,
                    password: param.password,
                    oidc_claims_sub: param.oidc_claims_sub,
                    oidc_claims_name: param.oidc_claims_name,
                    oidc_claims_email: param.oidc_claims_email,
                    github_claims_sub: param.github_claims_sub,
                    github_claims_name: param.github_claims_name,
                    github_claims_email: param.github_claims_email,
                    google_claims_sub: param.google_claims_sub,
                    google_claims_name: param.google_claims_name,
                    google_claims_email: param.google_claims_email,
                    ethers_address: param.ethers_address,
                    lang: param.lang,
                };
                if user.base.id.is_some() {
                    save_param.id = user.base.id;
                }
                match self.save(save_param).await {
                    std::result::Result::Ok(id) => {
                        if id > 0 {
                            Ok(())
                        } else {
                            Err(anyhow::Error::msg("Failed to save user"))
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            std::result::Result::Ok(None) => {
                let save_param = SaveUserRequest {
                    id: None,
                    name: param.name,
                    email: param.email,
                    phone: param.phone,
                    password: param.password,
                    oidc_claims_sub: param.oidc_claims_sub,
                    oidc_claims_name: param.oidc_claims_name,
                    oidc_claims_email: param.oidc_claims_email,
                    github_claims_sub: param.github_claims_sub,
                    github_claims_name: param.github_claims_name,
                    github_claims_email: param.github_claims_email,
                    google_claims_sub: param.google_claims_sub,
                    google_claims_name: param.google_claims_name,
                    google_claims_email: param.google_claims_email,
                    ethers_address: param.ethers_address,
                    lang: param.lang,
                };
                match self.save(save_param).await {
                    std::result::Result::Ok(id) => {
                        if id > 0 {
                            Ok(())
                        } else {
                            Err(anyhow::Error::msg("Failed to save user, because no found user"))
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }

    #[audit_log("[USER][FIND] name: {param.name.clone().unwrap_or_default()}")]
    async fn find(&self, param: QueryUserRequest, page: PageRequest) -> Result<(PageResponse, Vec<User>), Error> {
        let repo = self.state.user_repo.lock().await;
        repo.get(&self.state.config).select(param.to_user(), page).await
    }

    #[audit_log("[USER][ADD] name: {param.name.clone().unwrap_or_default()}")]
    async fn save(&self, param: SaveUserRequest) -> Result<i64, Error> {
        let repo = self.state.user_repo.lock().await;
        if param.id.is_some() {
            repo.get(&self.state.config).update(param.to_user()).await
        } else {
            repo.get(&self.state.config).insert(param.to_user()).await
        }
    }

    #[audit_log("[USER][DELETE] id: {param.id}")]
    async fn delete(&self, param: DeleteUserRequest) -> Result<u64, Error> {
        let repo = self.state.user_repo.lock().await;
        repo.get(&self.state.config).delete_by_id(param.id).await
    }
}
