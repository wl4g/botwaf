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

use crate::{cache::redis::StringRedisCache, config::config, waf::ipfilter_redis::RedisIPFilter};
use anyhow::{Error, Result};
use botwaf_types::forwarder::HttpIncomingRequest;
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

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

lazy_static! {
    /// RwLock Notices:
    /// Read lock (shared lock):
    /// Allow multiple threads to hold read locks at the same time.
    /// Read locks will not block each other, and all read lock holders can read data concurrently.
    /// Write lock (exclusive lock):
    /// Only one thread can hold a write lock.
    /// The write lock will block all other threads' read-write lock requests until the write lock is released.
    /// Mutual exclusion rules for read-write locks:
    /// When a thread holds a write lock, other threads cannot obtain read or write locks. ---Therefore, the phantom read problem of the database is avoided
    /// When one or more threads hold a read lock, other threads cannot obtain a write lock. ---Therefore, RwLock is only suitable for scenarios with more reads and less writes, such as cache systems, configuration file reading, etc.
    static ref SINGLE_INSTANCE: RwLock<IPFilterManager> = RwLock::new(IPFilterManager::new());
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
        let redis_cache = Arc::new(StringRedisCache::new(&config::get_config().cache.redis));
        let handler = RedisIPFilter::new(
            redis_cache,
            config::get_config().services.blocked_header_name.to_owned(),
        );
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
        // If the read lock is poisoned, the program will panic.
        let this = IPFilterManager::get().read().unwrap();
        if let Some(implementation) = this.implementations.get(&name) {
            Ok(implementation.to_owned())
        } else {
            let errmsg = format!("Could not obtain registered IPFilter '{}'.", name);
            return Err(Error::msg(errmsg));
        }
    }
}
