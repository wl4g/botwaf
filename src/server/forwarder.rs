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

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{server::forwarder_http::HttpForwardHandler, types::forwarder::HttpIncomingRequest};
use anyhow::{Error, Result};
use async_trait::async_trait;
use axum::{body::Body, response::Response};
use lazy_static::lazy_static;

#[async_trait]
pub trait IForwarder {
    async fn init(&self) -> Result<()>;
    async fn http_forward(&self, incoming: Arc<HttpIncomingRequest>) -> Result<Response<Body>>;
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
    static ref SINGLE_INSTANCE: RwLock<ForwarderManager> = RwLock::new(ForwarderManager::new());
}

pub struct ForwarderManager {
    pub implementations: HashMap<String, Arc<dyn IForwarder + Send + Sync>>,
}

impl ForwarderManager {
    fn new() -> Self {
        ForwarderManager {
            implementations: HashMap::new(),
        }
    }

    pub fn get() -> &'static RwLock<ForwarderManager> {
        &SINGLE_INSTANCE
    }

    pub async fn init() {
        tracing::info!("Register Botwaf Http IForwarder ...");
        match Self::get()
            .write() // If acquire fails, then it block until acquired.
            .unwrap() // If acquire fails, then it should panic.
            .register(HttpForwardHandler::NAME.to_owned(), HttpForwardHandler::new())
        {
            Ok(registered) => {
                tracing::info!("Initializing Botwaf Http IForwarder ...");
                let _ = registered.init().await;
            }
            Err(e) => panic!("Failed to register Http IForwarder: {}", e),
        }
    }

    fn register<T: IForwarder + Send + Sync + 'static>(
        &mut self,
        name: String,
        handler: Arc<T>,
    ) -> Result<Arc<T>, Error> {
        // Check if the name already exists
        if self.implementations.contains_key(&name) {
            let errmsg = format!("Forwarder handler Factory: Name '{}' already exists", name);
            return Err(Error::msg(errmsg));
        }
        self.implementations.insert(name, handler.to_owned());
        Ok(handler)
    }

    pub fn get_implementation(name: String) -> Result<Arc<dyn IForwarder + Send + Sync>, Error> {
        // If the read lock is poisoned, the program will panic.
        let this = ForwarderManager::get().read().unwrap();
        if let Some(implementation) = this.implementations.get(&name) {
            Ok(implementation.to_owned())
        } else {
            let errmsg = format!("Could not obtain registered Forwarder '{}'.", name);
            return Err(Error::msg(errmsg));
        }
    }
}
