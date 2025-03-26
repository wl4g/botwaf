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

use crate::sys::handler::user_handler::UserHandler;
use crate::util::auths::SecurityContext;
use crate::util::web::ValidatedJson;
use crate::{context::state::BotwafState, sys::handler::user_handler::IUserHandler};
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use botwaf_types::user::{DeleteUserRequest, QueryUserRequest, SaveUserRequest, User};
use botwaf_types::{
    user::{DeleteUserResponse, QueryUserResponse, SaveUserRequestWith, SaveUserResponse},
    PageRequest, RespBase,
};

pub fn init() -> Router<BotwafState> {
    Router::new()
        .route("/sys/user/current", get(handle_get_current_user))
        .route("/sys/user/current", post(handle_post_current_user))
        .route("/sys/user/query", get(handle_query_users))
        .route("/sys/user/save", post(handle_save_user))
        .route("/sys/user/delete", post(handle_delete_user))
}

#[utoipa::path(
    get,
    path = "/sys/user/current",
    responses((status = 200, description = "Getting for current user.", body = User)),
    tag = "User"
)]
async fn handle_get_current_user(State(state): State<BotwafState>) -> impl IntoResponse {
    let cur_user = SecurityContext::get_instance().get().await;
    tracing::info!("Getting for current user: {:?}", cur_user);

    let cur_user_uid = cur_user.map(|u| u.uid);
    match get_user_handler(&state)
        .get(cur_user_uid, None, None, None, None, None, None, None)
        .await
    {
        Ok(result) => match result {
            Some(user) => Ok(Json(user)),
            None => Err(StatusCode::NO_CONTENT),
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/sys/user/current",
    request_body = SaveUserRequestWith,
    responses((status = 200, description = "Configure for current user.", body = RespBase)),
    tag = "User"
)]
async fn handle_post_current_user(
    State(state): State<BotwafState>,
    ValidatedJson(param): ValidatedJson<SaveUserRequestWith>,
) -> impl IntoResponse {
    let cur_user = SecurityContext::get_instance().get().await;
    tracing::info!("Configure for current user: {:?}", cur_user);

    let cur_user_uid = cur_user.map(|u| u.uid);
    match get_user_handler(&state)
        .set(cur_user_uid, None, None, None, None, None, None, None, param)
        .await
    {
        Ok(_) => (StatusCode::OK, RespBase::success().to_json()).into_response(),
        Err(e) => (StatusCode::OK, RespBase::error(e).to_json()).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/sys/user/query",
    params(QueryUserRequest, PageRequest),
    responses((status = 200, description = "Getting for all users.", body = QueryUserResponse)),
    tag = "User"
)]
async fn handle_query_users(
    State(state): State<BotwafState>,
    Query(param): Query<QueryUserRequest>,
    Query(page): Query<PageRequest>,
) -> impl IntoResponse {
    match get_user_handler(&state).find(param, page).await {
        Ok((page, data)) => Ok(Json(QueryUserResponse::new(page, data))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/sys/user/save",
    request_body = SaveUserRequest,
    responses((status = 200, description = "Save for user.", body = SaveUserResponse)),
    tag = "User"
)]
async fn handle_save_user(
    State(state): State<BotwafState>,
    ValidatedJson(param): ValidatedJson<SaveUserRequest>,
) -> impl IntoResponse {
    match get_user_handler(&state).save(param).await {
        Ok(result) => Ok(Json(SaveUserResponse::new(result))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/sys/user/delete",
    request_body = DeleteUserRequest,
    responses((status = 200, description = "Delete for user.", body = DeleteUserResponse)),
    tag = "User"
)]
async fn handle_delete_user(
    State(state): State<BotwafState>,
    Json(param): Json<DeleteUserRequest>,
) -> impl IntoResponse {
    match get_user_handler(&state).delete(param).await {
        Ok(result) => Ok(Json(DeleteUserResponse::new(result))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn get_user_handler(state: &BotwafState) -> Box<dyn IUserHandler + '_> {
    Box::new(UserHandler::new(state))
}
