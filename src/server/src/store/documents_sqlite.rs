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

use anyhow::{ Error, Ok };
use axum::async_trait;

use crate::config::config::DbProperties;
use server_types::document::Document;
use server_types::PageRequest;
use server_types::PageResponse;
use super::AsyncRepository;
use super::sqlite::SQLiteRepository;

pub struct DocumentSQLiteRepository {
    inner: SQLiteRepository<Document>,
}

impl DocumentSQLiteRepository {
    pub async fn new(config: &DbProperties) -> Result<Self, Error> {
        Ok(DocumentSQLiteRepository {
            inner: SQLiteRepository::new(config).await?,
        })
    }
}

#[async_trait]
impl AsyncRepository<Document> for DocumentSQLiteRepository {
    async fn select(
        &self,
        document: Document,
        page: PageRequest
    ) -> Result<(PageResponse, Vec<Document>), Error> {
        let result = dynamic_sqlite_query!(
            document,
            "documents",
            self.inner.get_pool(),
            "update_time",
            page,
            Document
        ).unwrap();

        tracing::info!("query documents: {:?}", result);
        Ok((result.0, result.1))
    }

    async fn select_by_id(&self, id: i64) -> Result<Document, Error> {
        let document = sqlx
            ::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1")
            .bind(id)
            .fetch_one(self.inner.get_pool()).await
            .unwrap();

        tracing::info!("query document: {:?}", document);
        Ok(document)
    }

    async fn insert(&self, mut document: Document) -> Result<i64, Error> {
        let inserted_id = dynamic_sqlite_insert!(
            document,
            "documents",
            self.inner.get_pool()
        ).unwrap();
        tracing::info!("Inserted document.id: {:?}", inserted_id);
        Ok(inserted_id)
    }

    async fn update(&self, mut document: Document) -> Result<i64, Error> {
        let updated_id = dynamic_sqlite_update!(
            document,
            "documents",
            self.inner.get_pool()
        ).unwrap();
        tracing::info!("Updated document.id: {:?}", updated_id);
        Ok(updated_id)
    }

    async fn delete_all(&self) -> Result<u64, Error> {
        let delete_result = sqlx
            ::query("DELETE FROM documents")
            .execute(self.inner.get_pool()).await
            .unwrap();

        tracing::info!("Deleted result: {:?}", delete_result);
        Ok(delete_result.rows_affected())
    }

    async fn delete_by_id(&self, id: i64) -> Result<u64, Error> {
        let delete_result = sqlx
            ::query("DELETE FROM documents WHERE id = $1")
            .bind(id)
            .execute(self.inner.get_pool()).await
            .unwrap();

        tracing::info!("Deleted result: {:?}", delete_result);
        Ok(delete_result.rows_affected())
    }
}
