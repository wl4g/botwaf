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

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use std::{collections::HashMap, sync::Arc};

#[derive(Serialize, Deserialize)]
pub struct HttpThreatSampleRecord {
    content: String,
    label: String,
    metadata: serde_json::Value,
}

#[derive(
    Deserialize, Clone, Debug, PartialEq, utoipa::ToSchema,
)]
pub enum KnowledgeStatus {
    RECEIVED,
    //QUEUED,
    EMBEDDING,
    EMBEDDED,
    FAILED,
}

#[derive(
    Deserialize, Clone, Debug, PartialEq, utoipa::ToSchema,
)]
pub struct KnowledgeUploadFile {
    pub id: String,
    pub name: String,
    pub labels: HashMap<String, String>,
    pub positive: bool,
    pub status: KnowledgeStatus,
    pub create_at: u64,
    pub create_by: Option<String>,
}

impl KnowledgeUploadFile {
    pub async fn new(
        name: String,
        labels: HashMap<String, String>,
        positive: bool,
        create_by: Option<String>,
    ) -> Arc<Self> {
        Arc::new(KnowledgeUploadFile {
            id: Uuid::new_v4().to_string().replace("-", ""),
            name,
            labels,
            positive,
            status: KnowledgeStatus::RECEIVED,
            create_at: Utc::now().timestamp_millis() as u64,
            create_by,
        })
    }
}
