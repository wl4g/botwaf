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

use crate::{
    cache::redis::StringRedisCache, config::config, server::ipfilter_redis::RedisIPFilter,
    types::server::HttpIncomingRequest,
};
use anyhow::{Error, Result};
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

lazy_static! {
    /// RwLock 的锁机制规则:
    ///   读锁(共享锁):
    ///     允许多个线程同时持有读锁。
    ///     读锁之间不会互相阻塞，所有读锁持有者可以并发读取数据。
    ///   写锁(独占锁):
    ///     只有一个线程可以持有写锁。
    ///     写锁会阻塞所有其他线程的读锁和写锁请求，直到写锁释放。
    ///   读写锁的互斥规则:
    ///     当一个线程持有写锁时，其他线程无法获取读锁或写锁。
    ///     当一个或多个线程持有读锁时，其他线程无法获取写锁。
    static ref SINGLE_INSTANCE: RwLock<IPFilterManager> = RwLock::new(IPFilterManager::new());
}

#[async_trait::async_trait]
pub trait IPFilter {
    /// Initialization.
    async fn init(&self) -> Result<(), Error>;

    /// Checks if an IP address is in the blacklist
    async fn is_blocked(&self, incoming: Arc<HttpIncomingRequest>) -> Result<bool, Error>;

    /// Adds an IP address to the blacklist
    async fn block_ip(&self, incoming: Arc<HttpIncomingRequest>) -> Result<bool, Error>;

    /// Removes an IP address from the blacklist
    async fn unblock_ip(&self, incoming: Arc<HttpIncomingRequest>) -> Result<bool, Error>;
}

pub struct IPFilterManager {
    pub implementations: HashMap<String, Arc<dyn IPFilter + Send + Sync>>,
}

impl IPFilterManager {
    fn new() -> Self {
        IPFilterManager {
            implementations: HashMap::new(),
        }
    }

    pub fn get() -> &'static RwLock<IPFilterManager> {
        &SINGLE_INSTANCE
    }

    pub async fn init() {
        tracing::info!("Register Botwaf Redis IPFilter ...");
        let redis_cache = Arc::new(StringRedisCache::new(&config::CFG.cache.redis));
        let handler = RedisIPFilter::new(redis_cache, config::CFG.botwaf.blocked_header_name.to_owned());
        match Self::get()
            .write() // If acquire fails, then it block until acquired.
            .unwrap() // If acquire fails, then it should panic.
            .register(RedisIPFilter::NAME.to_owned(), handler)
        {
            Ok(registered) => {
                tracing::info!("Initializing Botwaf Redis IPFilter ...");
                let _ = registered.init().await;
            }
            Err(e) => panic!("Failed to register Redis IPFilter: {}", e),
        }
    }

    fn register<T: IPFilter + Send + Sync + 'static>(
        &mut self,
        name: String,
        handler: Arc<T>,
    ) -> Result<Arc<T>, Error> {
        // Check if the name already exists
        if self.implementations.contains_key(&name) {
            let errmsg = format!("Updater handler Factory: Name '{}' already exists", name);
            return Err(Error::msg(errmsg));
        }
        self.implementations.insert(name, handler.to_owned());
        Ok(handler)
    }

    pub fn get_implementation(name: String) -> Result<Arc<dyn IPFilter + Send + Sync>, Error> {
        let this = IPFilterManager::get().read().unwrap(); // If the read lock is poisoned, the program will panic.
        if let Some(implementation) = this.implementations.get(&name) {
            Ok(implementation.to_owned())
        } else {
            let errmsg = format!("Could not obtain registered IPFilter '{}'.", name);
            return Err(Error::msg(errmsg));
        }
    }
}
