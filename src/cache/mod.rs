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

use std::collections::HashMap;

use anyhow::Error;
use async_trait::async_trait;

use crate::config::config::{AppConfigProperties, CacheProvider};

pub mod memory;
pub mod redis;

#[async_trait]
pub trait ICache<T>: Send + Sync {
    async fn get(&self, key: String) -> Result<Option<T>, Error>
    where
        T: 'static + Send + Sync;

    async fn set(&self, key: String, value: T, seconds: Option<i32>) -> Result<bool, Error>
    where
        T: 'static + Send + Sync;

    async fn set_nx(&self, key: String, value: Option<String>) -> Result<bool, Error>;

    async fn keys(&self, pattern: String) -> Result<Vec<String>, Error>
    where
        T: 'static + Send + Sync;

    async fn hget(&self, key: String, field: Option<String>) -> Result<Option<String>, Error>;

    async fn hget_all(&self, name: String) -> Result<Option<HashMap<String, String>>, Error>;

    async fn hkeys(&self, key: String) -> Result<Vec<String>, Error>
    where
        T: 'static + Send + Sync;

    async fn hset(&self, key: String, field_values: Option<Vec<(String, String)>>) -> Result<bool, Error>;

    async fn hset_nx(&self, key: String, field: String, value: String) -> Result<bool, Error>;

    async fn hdel(&self, key: String, field: String) -> Result<bool, Error>;

    async fn expire(&self, key: String, milliseconds: i64) -> Result<bool, Error>;

    async fn get_bit(&self, key: String, offset: u64) -> Result<bool, Error>;

    async fn set_bit(&self, key: String, offset: u64, value: bool) -> Result<bool, Error>;

    async fn del(&self, key: String) -> Result<bool, Error>;
}

pub struct CacheContainer<T>
where
    T: 'static + Send + Sync,
{
    memory_cache: Box<dyn ICache<T>>,
    redis_cache: Box<dyn ICache<T>>,
}

impl<T> CacheContainer<T>
where
    T: 'static + Send + Sync,
{
    pub fn new(memory_cache: Box<dyn ICache<T>>, redis_cache: Box<dyn ICache<T>>) -> Self {
        CacheContainer { memory_cache, redis_cache }
    }

    fn memory_cache(&self) -> &dyn ICache<T> {
        &*self.memory_cache
    }

    fn redis_cache(&self) -> &dyn ICache<T> {
        &*self.redis_cache
    }

    pub fn get(&self, config: &AppConfigProperties) -> &dyn ICache<T> {
        match config.cache.provider {
            CacheProvider::Memory => self.memory_cache(),
            CacheProvider::Redis => self.redis_cache(),
        }
    }
}
