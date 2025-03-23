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

use std::any::Any;
use std::marker::PhantomData;
use std::time::Duration;

use anyhow::Error;
use axum::async_trait;

use mongodb::options::{ ReadConcern, WriteConcern };
use mongodb::{ Client, Database, options::ClientOptions };

use super::AsyncRepository;
use crate::config::config::DbProperties;
use server_types::{ PageResponse, PageRequest };

pub struct MongoRepository<T: Any + Send + Sync> {
    phantom: PhantomData<T>,
    database: Database,
}

impl<T: Any + Send + Sync> MongoRepository<T> {
    pub async fn new(config: &DbProperties) -> Result<Self, Error> {
        let mut client_options = ClientOptions::parse(
            &config.mongo.url.to_owned().expect("Mongo url missing configured")
        ).await?;
        client_options.connect_timeout = Some(Duration::from_secs(10));
        client_options.server_selection_timeout = Some(Duration::from_secs(30));
        client_options.write_concern = Some(WriteConcern::majority()); // Reliable write
        // Notice: read concern level 'snapshot' is only valid in a transaction
        client_options.read_concern = Some(ReadConcern::local());

        let client = Client::with_options(client_options)?;
        let database = client.database(
            &config.mongo.database.to_owned().expect("Mongo database missing configured")
        );
        Ok(MongoRepository {
            phantom: PhantomData,
            database,
        })
    }

    pub fn get_database(&self) -> &Database {
        &self.database
    }
}

#[allow(unused)]
#[async_trait]
impl<T: Any + Send + Sync> AsyncRepository<T> for MongoRepository<T> {
    async fn select(
        &self,
        mut param: T,
        page: PageRequest
    ) -> Result<(PageResponse, Vec<T>), Error> {
        //use crate::dynamic_mongo_query;
        //match dynamic_mongo_query!(param, self.database.collection("users"), "update_time", page, User) {
        //    Ok(result) => {
        //        // tracing::info!("query users: {:?}", result);
        //        Ok((result.0, result.1))
        //    }
        //    Err(error) => Err(error),
        //}
        unimplemented!("select not implemented for MongoRepository")
    }

    async fn select_by_id(&self, id: i64) -> Result<T, Error> {
        unimplemented!("select_by_id not implemented for MongoRepository")
    }

    async fn insert(&self, param: T) -> Result<i64, Error> {
        unimplemented!("insert not implemented for MongoRepository")
    }

    async fn update(&self, param: T) -> Result<i64, Error> {
        unimplemented!("update not implemented for MongoRepository")
    }

    async fn delete_all(&self) -> Result<u64, Error> {
        unimplemented!("delete_all not implemented for MongoRepository")
    }

    async fn delete_by_id(&self, id: i64) -> Result<u64, Error> {
        unimplemented!("delete_by_id not implemented for MongoRepository")
    }
}

#[macro_export]
macro_rules! dynamic_mongo_query {
    ($bean:expr, $collection:expr, $order_by:expr, $page:expr, $($t:ty),+) => {
        {
            use mongodb::bson::{doc, Document};
            use futures::stream::TryStreamExt;

            let serialized = serde_json::to_value(&$bean).unwrap();
            let obj = serialized.as_object().unwrap();

            let mut filter = Document::new();
            for (key, value) in obj {
                if !value.is_null() {
                    let v = value.as_str().unwrap_or("");
                    if !v.is_empty() {
                        filter.insert(key, v);
                    }
                }
            }
            if let Some(id) = $bean.base.id {
                filter.insert("id", id);
            }

            let options = mongodb::options::FindOptions::builder()
                .skip($page.get_offset() as u64)
                .limit($page.get_limit() as i64)
                .sort(doc! { $order_by: -1 })
                .build();

            // Queries to get total count.
            let total_count = $collection.count_documents(filter.clone()).await.unwrap();

            // Queries to get data.
            let cursor = $collection
                .find(filter)
                .skip(options.skip.unwrap())
                .limit(options.limit.unwrap())
                .sort(options.sort.unwrap()).await?;

            match cursor.try_collect().await {
                std::result::Result::Ok(result) => {
                  let page = PageResponse::new(
                      Some(total_count as i64),
                      Some($page.get_offset()),
                      Some($page.get_limit()));
                    Ok((page, result))
                },
                Err(error) => {
                    Err(error.into())
                }
            }
        }
    };
}

#[macro_export]
macro_rules! dynamic_mongo_insert {
    ($bean:expr, $collection:expr) => {
        {
            use crate::utils::auths::SecurityContext;

            let insert_by = SecurityContext::get_instance().get_current_uname_for_store().await;
            let id = $bean.base.pre_insert(insert_by).await;
            //use mongodb::bson::to_bson;
            //let serialized = to_bson(&$bean)?;
            //let obj = serialized.as_document().unwrap().clone();

            // Notice:
            // 1. (SQLite) Because the ORM library is not used for the time being, the fields are dynamically
            // parsed based on serde_json, so the #[serde(rename="xx")] annotation is effective.
            // 2. (MongoDB) The underlying BSON serialization is also based on serde, so using #[serde(rename="xx")] is also valid
            // TODO: It is recommended to use an ORM framework, see: https://github.com/diesel-rs/diesel
            let result = $collection.insert_one(&$bean).await?;

            if let Some(inserted_id) = result.inserted_id.as_object_id() {
                tracing::debug!("inserted_id: {}", inserted_id);
                Ok(id)
            } else {
                Ok(-1)
            }
        }
    };
}

#[macro_export]
macro_rules! dynamic_mongo_update {
    ($bean:expr, $collection:expr) => {
        {
            use mongodb::bson::{doc, to_bson, Bson};
            use crate::utils::auths::SecurityContext;

            let update_by = SecurityContext::get_instance().get_current_uname_for_store().await;
            $bean.base.pre_update(update_by).await;

            // Note:
            // 1. (SQLite) Because the ORM library is not used for the time being, the fields are dynamically
            // parsed based on serde_json, so the #[serde(rename="xx")] annotation is effective.
            // 2. (MongoDB) The underlying BSON serialization is also based on serde, so using #[serde(rename="xx")] is also valid
            // TODO: It is recommended to use an ORM framework, see: https://github.com/diesel-rs/diesel
            let id = $bean.base.id.unwrap();
            let serialized = to_bson(&$bean)?;
            let obj = serialized.as_document().unwrap().clone();

            fn is_empty_value(value: &Bson) -> bool {
                match value {
                    Bson::String(s) => s.is_empty(),
                    Bson::Array(arr) => arr.is_empty(),
                    Bson::Document(doc) => doc.is_empty(),
                    Bson::Null => true,
                    _ => false,
                }
            }

            let mut update_doc = mongodb::bson::Document::new();
            for (key, value) in obj.iter() {
                if !is_empty_value(value) {
                    update_doc.insert(key, value.clone());
                }
            }

            let filter = doc! { "id": id };
            let update = doc! { "$set": update_doc };
            let result = $collection.update_one(filter, update).await?;

            if result.modified_count > 0 {
                Ok(id)
            } else {
                Ok(-1)
            }
        }
    };
}
