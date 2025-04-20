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
        let result = dynamic_postgres_query!(user, "users", self.inner.get_pool(), "update_time", page, User).unwrap();
        info!("query users: {:?}", result);
        Ok((result.0, result.1))
    }

    async fn select_by_id(&self, id: i64) -> Result<User, Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(self.inner.get_pool())
            .await
            .unwrap();

        info!("query user: {:?}", user);
        Ok(user)
    }

    async fn insert(&self, mut user: User) -> Result<i64, Error> {
        // use sqlx::Row;
        // let inserted_id = sqlx::query(
        //     r#"
        //     INSERT INTO users (id, name, email, password, create_by, create_time, update_by, update_time, del_flag)
        //     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id
        //     "#,
        // )
        // .bind(user.base.id)
        // .bind(user.name)
        // .bind(user.email)
        // .bind(user.password)
        // .bind(user.base.create_by)
        // .bind(user.base.create_time)
        // .bind(user.base.update_by)
        // .bind(user.base.update_time)
        // .bind(user.base.del_flag)
        // .fetch_one(self.inner.get_pool())
        // .await
        // .map(|row: sqlx::postgres::PgRow| row.get("id"))?;

        let inserted_id = dynamic_postgres_insert!(user, "users", self.inner.get_pool()).unwrap();
        info!("Inserted user.id: {:?}", inserted_id);
        Ok(inserted_id)
    }

    async fn update(&self, mut user: User) -> Result<i64, Error> {
        let updated_id = dynamic_postgres_update!(user, "users", self.inner.get_pool()).unwrap();
        info!("Updated user.id: {:?}", updated_id);
        Ok(updated_id)
    }

    async fn delete_all(&self) -> Result<u64, Error> {
        let delete_result = sqlx::query("DELETE FROM users")
            .execute(self.inner.get_pool())
            .await
            .unwrap();

        info!("Deleted result: {:?}", delete_result);
        Ok(delete_result.rows_affected())
    }

    async fn delete_by_id(&self, id: i64) -> Result<u64, Error> {
        let delete_result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(self.inner.get_pool())
            .await
            .unwrap();

        info!("Deleted result: {:?}", delete_result);
        Ok(delete_result.rows_affected())
    }
}
