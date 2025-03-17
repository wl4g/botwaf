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

use crate::types::knowledge::KnowledgeUploadFile;

use super::server::BotWafState;
use axum::{extract::State, response::IntoResponse, routing::post, Json, Router};
use hyper::StatusCode;

pub fn init() -> Router<BotWafState> {
    Router::new().route("/modules/knowledge/upload", post(handle_knowledge_upload))
}

#[utoipa::path(
    post,
    path = "/modules/knowledge/upload",
    request_body = KnowledgeUploadFile,
    responses((status = 200, description = "Upload Knowledge.", body = KnowledgeUploadFile)),
    tag = "Knowledge"
)]
async fn handle_knowledge_upload(
    State(state): State<BotWafState>,
    Json(param): Json<KnowledgeUploadFile>,
) -> impl IntoResponse {
    StatusCode::INTERNAL_SERVER_ERROR
}
