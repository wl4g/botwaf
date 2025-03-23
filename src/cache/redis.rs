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

use super::ICache;
use crate::config::config::RedisProperties;
use anyhow::Error;
use async_trait::async_trait;
use redis::{
    cluster::{ClusterClient, ClusterClientBuilder},
    cluster_async::ClusterConnection,
    RedisResult,
};
use std::{collections::HashMap, sync::Arc, time::Duration};

pub struct StringRedisCache {
    client: Arc<ClusterClient>,
}

impl StringRedisCache {
    pub fn new(config: &RedisProperties) -> Self {
        tracing::info!("Initializing redis with config: {:?}", config);

        let mut builder = ClusterClientBuilder::new(config.nodes.clone());
        if config.username.is_some() {
            builder = builder.username(config.username.clone().unwrap());
        }
        if config.password.is_some() {
            builder = builder.password(config.password.clone().unwrap());
        }
        if config.connection_timeout.is_some() {
            builder = builder.connection_timeout(Duration::from_millis(config.connection_timeout.clone().unwrap()));
        }
        if config.response_timeout.is_some() {
            builder = builder.response_timeout(Duration::from_millis(config.response_timeout.clone().unwrap()));
        }
        if config.retries.is_some() {
            builder = builder.retries(config.retries.clone().unwrap());
        }
        if config.max_retry_wait.is_some() {
            builder = builder.max_retry_wait(config.max_retry_wait.clone().unwrap());
        }
        if config.min_retry_wait.is_some() {
            builder = builder.min_retry_wait(config.min_retry_wait.clone().unwrap());
        }
        if config.read_from_replicas.is_some() {
            builder = builder.read_from_replicas();
        }
        let client = builder.build().expect("Failed to build redis cluster client");

        tracing::info!("Initialized the redis with config: {:?}", config);

        StringRedisCache {
            client: Arc::new(client),
        }
    }

    async fn get_async_connection(&self) -> Result<ClusterConnection, Error> {
        self.client.get_async_connection().await.map_err(Error::from)
    }
}

#[async_trait]
impl ICache<String> for StringRedisCache {
    async fn get(&self, key: String) -> Result<Option<String>, Error> {
        let mut con = self.get_async_connection().await?;
        let result: RedisResult<Option<String>> = redis::cmd("GET").arg(key).query_async(&mut con).await;
        Ok(result?)
    }

    async fn set(&self, key: String, value: String, seonds: Option<i32>) -> Result<bool, Error> {
        let mut con = self.get_async_connection().await?;
        let result: RedisResult<String> = if let Some(seconds) = seonds {
            redis::cmd("SETEX")
                .arg(key)
                .arg(seconds)
                .arg(value)
                .query_async(&mut con)
                .await
        } else {
            redis::cmd("SET").arg(key).arg(value).query_async(&mut con).await
        };
        Ok(result.map(|s| s == "OK")?)
    }

    async fn set_nx(&self, key: String, value: Option<String>) -> Result<bool, Error> {
        let mut con = self.get_async_connection().await?;
        let result: RedisResult<i64> = redis::cmd("SETNX").arg(key).arg(value).query_async(&mut con).await;
        Ok(result.map(|s| s > 0)?)
    }

    async fn keys(&self, pattern: String) -> Result<Vec<String>, Error> {
        let mut con = self.get_async_connection().await?;
        let result: RedisResult<Vec<String>> = redis::cmd("KEYS").arg(pattern).query_async(&mut con).await;
        Ok(result?)
    }

    async fn hget(&self, key: String, field: Option<String>) -> Result<Option<String>, Error> {
        let mut con = self.get_async_connection().await?;
        let result = redis::cmd("HGET")
            .arg(key)
            .arg(field.unwrap_or_default())
            .query_async(&mut con)
            .await;
        Ok(result?)
    }

    async fn hget_all(&self, key: String) -> Result<Option<HashMap<String, String>>, Error> {
        let mut con = self.get_async_connection().await?;
        let result: RedisResult<Option<HashMap<String, String>>> =
            redis::cmd("HGETALL").arg(key).query_async(&mut con).await;
        Ok(result?)
    }

    async fn hkeys(&self, key: String) -> Result<Vec<String>, Error> {
        let mut con = self.get_async_connection().await?;
        let result: RedisResult<Vec<String>> = redis::cmd("HKEYS").arg(key).query_async(&mut con).await;
        Ok(result?)
    }

    async fn hset(&self, key: String, field_values: Option<Vec<(String, String)>>) -> Result<bool, Error> {
        let mut con = self.get_async_connection().await?;
        let mut cmd = redis::cmd("HSET");
        cmd.arg(key);
        if let Some(fvs) = field_values {
            for (field, value) in fvs {
                cmd.arg(field).arg(value);
            }
        }
        let result: RedisResult<i64> = cmd.query_async(&mut con).await;
        Ok(result.map(|s| s >= 1)?)
    }

    async fn hset_nx(&self, key: String, field: String, value: String) -> Result<bool, Error> {
        let mut con = self.get_async_connection().await?;
        let mut cmd = redis::cmd("HSETNX");
        cmd.arg(key).arg(field).arg(value);
        let result: RedisResult<i64> = cmd.query_async(&mut con).await;
        Ok(result.map(|s| s > 0)?)
    }

    async fn hdel(&self, key: String, field: String) -> Result<bool, Error> {
        let mut con = self.get_async_connection().await?;
        let mut cmd = redis::cmd("HDEL");
        cmd.arg(key);
        cmd.arg(field);
        let result: RedisResult<i64> = cmd.query_async(&mut con).await;
        Ok(result.map(|s| s >= 1)?)
    }

    async fn expire(&self, key: String, milliseconds: i64) -> Result<bool, Error> {
        let mut con = self.get_async_connection().await?;
        let mut cmd = redis::cmd("PEXPIRE");
        cmd.arg(key);
        cmd.arg(milliseconds);
        cmd.arg("NX");
        let result: RedisResult<i64> = cmd.query_async(&mut con).await;
        Ok(result.map(|s| s > 0)?)
    }

    async fn get_bit(&self, key: String, offset: u64) -> Result<bool, Error> {
        let mut con = self.get_async_connection().await?;
        let mut cmd = redis::cmd("GETBIT");
        cmd.arg(key);
        cmd.arg(offset);
        let result: RedisResult<i64> = cmd.query_async(&mut con).await;
        Ok(result.map(|s| s >= 1)?)
    }

    async fn set_bit(&self, key: String, offset: u64, value: bool) -> Result<bool, Error> {
        let mut con = self.get_async_connection().await?;
        let mut cmd = redis::cmd("SETBIT");
        cmd.arg(key);
        cmd.arg(offset);
        cmd.arg(match value {
            true => 1,
            false => 0,
        });
        let result: RedisResult<i64> = cmd.query_async(&mut con).await;
        Ok(result.map(|s| s >= 1).unwrap_or(false))
    }

    async fn del(&self, key: String) -> Result<bool, Error> {
        let mut con = self.get_async_connection().await?;
        let result: RedisResult<i32> = redis::cmd("DEL").arg(key).query_async(&mut con).await;
        Ok(result.map(|n| n > 0).unwrap_or(false))
    }
}
