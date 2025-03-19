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

use std::{collections::HashMap, sync::Arc};

use anyhow::Error;
use async_trait::async_trait;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{config::config, updater::updater_langchain::SimpleLLMUpdater};

#[async_trait]
pub trait IBotwafUpdater: Send + Sync {
    async fn init(&self);
}

lazy_static! {
    static ref SINGLE_INSTANCE: Mutex<BotwafUpdaterManager> = Mutex::new(BotwafUpdaterManager::new());
}

pub struct BotwafUpdaterManager {
    pub implementations: HashMap<String, Arc<dyn IBotwafUpdater + Send + Sync>>,
}

impl BotwafUpdaterManager {
    fn new() -> Self {
        BotwafUpdaterManager {
            implementations: HashMap::new(),
        }
    }

    pub fn get() -> &'static Mutex<BotwafUpdaterManager> {
        &SINGLE_INSTANCE
    }

    pub async fn init() {
        tracing::info!("Register All Botwaf updaters ...");

        for config in &config::CFG.botwaf.updaters {
            if config.kind == SimpleLLMUpdater::KIND {
                tracing::info!("Initializing implementation Botwaf updater: {}", config.name);
                let handler = SimpleLLMUpdater::new(config).await;
                if let Err(e) = BotwafUpdaterManager::get()
                    .lock()
                    .await
                    .register(config.name.to_owned(), handler.clone())
                {
                    tracing::error!("Failed to register updater: {}", e);
                }
                tracing::info!("Registered implementation updater: {}", config.name);
            }
        }

        tracing::info!("Initializing All Botwaf updaters ...");
        for config in &config::CFG.botwaf.updaters {
            match Self::get()
                .lock()
                .await
                .get_implementation(config.name.to_owned())
                .await
            {
                Ok(handler) => {
                    tracing::info!("Initializing implementation Botwaf updater: {}", config.name);
                    handler.init().await;
                }
                Err(_) => {}
            }
        }
    }

    fn register(&mut self, name: String, handler: Arc<dyn IBotwafUpdater + Send + Sync>) -> Result<(), Error> {
        // Check if the name already exists
        if self.implementations.contains_key(&name) {
            let errmsg = format!("Updater Factory: Name '{}' already exists", name);
            return Err(Error::msg(errmsg));
        }
        self.implementations.insert(name, handler);
        Ok(())
    }

    pub async fn get_implementation(&self, name: String) -> Result<Arc<dyn IBotwafUpdater + Send + Sync>, Error> {
        if let Some(implementation) = self.implementations.get(&name) {
            Ok(implementation.clone())
        } else {
            let errmsg = format!("Handler Factory: Name '{}' does't exists", name);
            return Err(Error::msg(errmsg));
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BotWafAccessEvent {
    // Request information.
    pub method: String,
    pub scheme: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub headers: Option<HashMap<String, Option<String>>>,
    pub path: String,
    pub query: Option<String>,
    pub body: Option<String>,
    // Additional request information.
    pub req_id: Option<String>,
    pub client_ip: Option<String>,
    pub start_time: u64,
    // Response information.
    pub resp_status_code: Option<i32>,
    pub resp_headers: Option<HashMap<String, Option<String>>>,
    pub resp_body: Option<String>,
    // Additional response information.
    pub duration: Option<u64>,
}
