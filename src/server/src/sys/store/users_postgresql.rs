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

use crate::config::config::PostgresAppDBProperties;
use crate::dynamic_postgres_insert;
use crate::dynamic_postgres_query;
use crate::dynamic_postgres_update;
use crate::store::postgres::PostgresRepository;
use crate::store::AsyncRepository;
use anyhow::{Error, Ok};
use async_trait::async_trait;
use botwaf_types::sys::user::User;
use botwaf_types::PageRequest;
use botwaf_types::PageResponse;
use common_telemetry::info;

pub struct UserPostgresRepository {
    inner: PostgresRepository<User>,
}

impl UserPostgresRepository {
    pub async fn new(config: &PostgresAppDBProperties) -> Result<Self, Error> {
        Ok(UserPostgresRepository {
            inner: PostgresRepository::new(config).await?,
        })
    }
}

#[async_trait]
impl AsyncRepository<User> for UserPostgresRepository {
    async fn select(&self, user: User, page: PageRequest) -> Result<(PageResponse, Vec<User>), Error> {
        // use chrono::{DateTime, Utc};
        // use botwaf_utils::types::GenericValue;
        // let table = "sys_user";
        // let order_by = "update_time";
        // // Notice:
        // // 1. (SQLite) Because the ORM library is not used for the time being, the fields are dynamically
        // // parsed based on serde_json, so the #[serde(rename="xx")] annotation is effective.
        // // 2. (MongoDB) The underlying BSON serialization is also based on serde, so using #[serde(rename="xx")] is also valid
        // // TODO: It is recommended to use an ORM framework, see: https://github.com/diesel-rs/diesel
        // let serialized = serde_json::to_value(&user)?;
        // let obj = serialized
        //     .as_object()
        //     .ok_or_else(|| anyhow::anyhow!("Expected JSON object"))?;
        // let mut fields = Vec::new();
        // let mut params = Vec::new();
        // let mut index = 0;
        // for (key, value) in obj {
        //     if !value.is_null() {
        //         let v = value.as_str().unwrap_or("");
        //         if !v.is_empty() {
        //             index += 1;
        //             // Notice: Must use $x expression? otherwise, the sqlx will not work.
        //             // e.g: "SELECT COUNT(1) as count FROM my_table WHERE create_time = $1 AND update_time = $2"
        //             fields.push(format!("{} = ${}", key, index));
        //             if key == "create_time" || key == "update_time" {
        //                 let dt = DateTime::parse_from_rfc3339(v)?;
        //                 params.push(GenericValue::DateTime(dt.with_timezone(&Utc)));
        //             } else {
        //                 params.push(GenericValue::String(v.to_string()));
        //             }
        //         }
        //     }
        // }
        // if let Some(id) = user.base.id {
        //     fields.push("id = ?".to_string());
        //     params.push(GenericValue::Int64(id));
        // }
        // let where_clause = if fields.is_empty() {
        //     "1=1".to_string()
        // } else {
        //     fields.join(" AND ")
        // };
        // // Queries to get total count.
        // let total_query = format!("SELECT COUNT(1) FROM {} WHERE {}", table, where_clause);
        // use sqlx::Row;
        // let mut total_operator = sqlx::query(&total_query);
        // for param in params.iter() {
        //     if let GenericValue::Bool(v) = param {
        //         total_operator = total_operator.bind(v);
        //     } else if let GenericValue::Int64(v) = param {
        //         total_operator = total_operator.bind(v);
        //     } else if let GenericValue::Float64(v) = param {
        //         total_operator = total_operator.bind(v);
        //     } else if let GenericValue::String(v) = param {
        //         total_operator = total_operator.bind(v);
        //     } else if let GenericValue::DateTime(v) = param {
        //         total_operator = total_operator.bind(v);
        //     }
        // }
        // let total_count = total_operator.fetch_one(self.inner.get_pool()).await?.get::<i64, _>(0);
        // // Queries to get data.
        // let query = format!(
        //     "SELECT * FROM {} WHERE {} ORDER BY {} LIMIT {} OFFSET {}",
        //     table,
        //     where_clause,
        //     order_by,
        //     page.get_limit(),
        //     page.get_offset()
        // );
        // let mut operator = sqlx::query_as::<_, User>(&query);
        // for param in params.iter() {
        //     if let GenericValue::Bool(v) = param {
        //         operator = operator.bind(v);
        //     } else if let GenericValue::Int64(v) = param {
        //         operator = operator.bind(v);
        //     } else if let GenericValue::Float64(v) = param {
        //         operator = operator.bind(v);
        //     } else if let GenericValue::String(v) = param {
        //         operator = operator.bind(v);
        //     } else if let GenericValue::DateTime(v) = param {
        //         operator = operator.bind(v);
        //     }
        // }
        // match operator.fetch_all(self.inner.get_pool()).await {
        //     std::result::Result::Ok(result) => {
        //         let p = PageResponse::new(Some(total_count), Some(page.get_offset()), Some(page.get_limit()));
        //         Ok((p, result))
        //     }
        //     Err(error) => Err(error.into()),
        // }

        let result = dynamic_postgres_query!(user, "users", self.inner.get_pool(), "update_time", page, User)?;
        info!("query users: {:?}", result);
        Ok((result.0, result.1))
    }

    async fn select_by_id(&self, id: i64) -> Result<User, Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 and del_flag = 0")
            .bind(id)
            .fetch_one(self.inner.get_pool())
            .await?;

        info!("query user: {:?}", user);
        Ok(user)
    }

    async fn insert(&self, mut user: User) -> Result<i64, Error> {
        // use chrono::{DateTime, Utc};
        // use botwaf_utils::types::GenericValue;
        // let serialized = serde_json::to_value(user).unwrap();
        // let obj = serialized.as_object().unwrap();
        // let mut fields = Vec::new();
        // let mut values = Vec::new();
        // let mut params = Vec::new();
        // for (key, value) in obj {
        //     if !value.is_null() {
        //         if value.is_boolean() {
        //             let v = value.as_bool().unwrap();
        //             fields.push(key.as_str());
        //             values.push("?");
        //             params.push(GenericValue::Bool(v));
        //         } else if value.is_number() {
        //             if value.is_i64() {
        //                 let v = value.as_i64().unwrap();
        //                 fields.push(key.as_str());
        //                 values.push("?");
        //                 params.push(GenericValue::Int64(v));
        //             } else if value.is_f64() {
        //                 let v = value.as_f64().unwrap();
        //                 fields.push(key.as_str());
        //                 values.push("?");
        //                 params.push(GenericValue::Float64(v));
        //             }
        //         } else if value.is_string() {
        //             let v = value.as_str().unwrap_or("");
        //             if !v.is_empty() {
        //                 fields.push(key.as_str());
        //                 values.push("?");
        //                 if key == "create_time" || key == "update_time" {
        //                     let dt = DateTime::parse_from_rfc3339(v)?;
        //                     params.push(GenericValue::DateTime(dt.with_timezone(&Utc)));
        //                 } else {
        //                     params.push(GenericValue::String(v.to_string()));
        //                 }
        //             }
        //         }
        //     }
        // }
        // let query = format!("INSERT INTO {} ({}) VALUES ({}) ON CONFLICT (id) DO UPDATE SET {} RETURNING id",
        //         $table, fields.join(","), values.join(","), "update_time = CURRENT_TIMESTAMP(13)");
        // let mut operator = sqlx::query(&query);
        // for param in params.iter() {
        //     if let GenericValue::Bool(v) = param {
        //         operator = operator.bind(v);
        //     } else if let GenericValue::Int64(v) = param {
        //         operator = operator.bind(v);
        //     } else if let GenericValue::Float64(v) = param {
        //         operator = operator.bind(v);
        //     } else if let GenericValue::String(v) = param {
        //         operator = operator.bind(v);
        //     } else if let GenericValue::DateTime(v) = param {
        //         operator = operator.bind(v);
        //     }
        // }

        let inserted_id = dynamic_postgres_insert!(user, "users", self.inner.get_pool())?;
        info!("Inserted user.id: {:?}", inserted_id);
        Ok(inserted_id)
    }

    async fn update(&self, mut user: User) -> Result<i64, Error> {
        let updated_id = dynamic_postgres_update!(user, "users", self.inner.get_pool())?;
        info!("Updated user.id: {:?}", updated_id);
        Ok(updated_id)
    }

    async fn delete_all(&self) -> Result<u64, Error> {
        let delete_result = sqlx::query("DELETE FROM users").execute(self.inner.get_pool()).await?;

        info!("Deleted result: {:?}", delete_result);
        Ok(delete_result.rows_affected())
    }

    async fn delete_by_id(&self, id: i64) -> Result<u64, Error> {
        let delete_result = sqlx::query("DELETE FROM users WHERE id = $1 and del_flag = 0")
            .bind(id)
            .execute(self.inner.get_pool())
            .await?;

        info!("Deleted result: {:?}", delete_result);
        Ok(delete_result.rows_affected())
    }
}
