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

pub mod api_v1;
pub mod modules;
pub mod sys;

use anyhow::Error;
use chrono::{DateTime, Utc};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use validator::Validate;

use botwaf_utils::snowflake::SnowflakeIdGenerator;
// use sqlx::{ Decode, FromRow };

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, FromRow, utoipa::ToSchema)]
pub struct BaseBean {
    #[schema(rename = "id")]
    pub id: Option<i64>,
    #[schema(rename = "status")]
    pub status: Option<i32>,
    #[sqlx(rename = "create_by")]
    #[schema(read_only = true)]
    // Notice: Since we are currently using serde serialization to implement custom ORM,
    // the #[serde(rename=xx)] rename will not only take effect on the restful APIs but
    // also on the DB, so for simplicity, we will unifed use underscores.
    //#[serde(rename = "createBy")]
    pub create_by: Option<String>,
    #[schema(read_only = true)]
    pub create_time: Option<DateTime<Utc>>,
    #[schema(read_only = true)]
    pub update_by: Option<String>,
    #[schema(read_only = true)]
    pub update_time: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub del_flag: Option<i32>,
}

impl BaseBean {
    pub fn new_empty() -> Self {
        Self {
            id: None,
            status: None,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
            del_flag: None,
        }
    }

    pub fn new_with_id(id: Option<i64>) -> Self {
        let now = Utc::now();
        Self {
            id,
            status: Some(0),
            create_by: None,
            create_time: Some(now),
            update_by: None,
            update_time: Some(now),
            del_flag: Some(0),
        }
    }

    pub fn new_with_by(id: Option<i64>, create_by: Option<String>, update_by: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id,
            status: Some(0),
            create_by,
            create_time: Some(now),
            update_by,
            update_time: Some(now),
            del_flag: Some(0),
        }
    }

    pub async fn pre_insert(&mut self, create_by: Option<String>) -> i64 {
        self.id = Some(SnowflakeIdGenerator::default_next_jssafe());
        self.create_by = create_by;
        self.create_time = Some(Utc::now());
        self.del_flag = Some(0);
        self.id.unwrap()
    }

    pub async fn pre_update(&mut self, update_by: Option<String>) {
        self.update_by = update_by;
        self.update_time = Some(Utc::now());
        self.del_flag = Some(0);
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Validate, utoipa::ToSchema, utoipa::IntoParams)]
pub struct PageRequest {
    #[schema(example = "1")]
    #[validate(range(min = 1, max = 1000))]
    pub num: Option<u32>, // page number.
    #[schema(example = "10")]
    #[validate(range(min = 1, max = 1000))]
    pub limit: Option<u32>, // The per page records count.
                            // For large data of fast-queries cached condition acceleration.
                            // pub cached_forward_last_min_id: Option<i64>,
                            // pub cached_backend_last_max_id: Option<i64>,
}

impl PageRequest {
    pub fn default() -> PageRequest {
        PageRequest {
            num: Some(1),
            limit: Some(10),
            // cached_forward_last_min_id: None,
            // cached_backend_last_max_id: None,
        }
    }
    pub fn get_offset(&self) -> u32 {
        let n = self.num.unwrap_or(1);
        if n < 1 {
            1
        } else {
            (n - 1) * self.get_limit()
        }
    }

    pub fn get_limit(&self) -> u32 {
        let l = self.limit.unwrap_or(10);
        if l < 1 {
            1
        } else {
            l
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, utoipa::ToSchema)]
pub struct PageResponse {
    pub total: Option<i64>, // The current conditions snapshot data of total records count.
    pub num: Option<u32>,   // page number.
    pub limit: Option<u32>, // The per page records count.
                            // For large data of fast-queries cached condition acceleration.
                            // pub cached_forward_last_min_id: Option<i64>,
                            // pub cached_backend_last_max_id: Option<i64>,
}

impl PageResponse {
    pub fn new(total: Option<i64>, num: Option<u32>, limit: Option<u32>) -> Self {
        Self { total, num, limit }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, Validate, utoipa::ToSchema, utoipa::IntoParams)]
pub struct RespBase {
    pub(crate) errcode: Option<i8>,
    pub(crate) errmsg: Option<String>,
}

#[allow(unused)]
impl RespBase {
    pub fn success() -> Self {
        Self {
            errcode: Some(0),
            errmsg: Some("ok".to_string()),
        }
    }

    pub fn error(e: Error) -> Self {
        Self {
            errcode: Some(StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i8),
            errmsg: Some(e.to_string()),
        }
    }

    #[allow(unused)]
    pub fn errmsg(errmsg: &str) -> Self {
        Self {
            errcode: Some(StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i8),
            errmsg: Some(errmsg.to_owned()),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
