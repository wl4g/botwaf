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
use tokio::sync::Mutex;

use crate::{config::config, updater::analytic_handler_llm::SimpleLlmAnalyticsHandler};

#[async_trait]
pub trait IAnalyticsHandler: Send + Sync {
    async fn start(&self);
}

lazy_static! {
    static ref SINGLE_INSTANCE: Mutex<AnalyticsHandlerFactory> = Mutex::new(AnalyticsHandlerFactory::new());
}

pub struct AnalyticsHandlerFactory {
    pub implementations: HashMap<String, Arc<dyn IAnalyticsHandler + Send + Sync>>,
}

impl AnalyticsHandlerFactory {
    fn new() -> Self {
        AnalyticsHandlerFactory {
            implementations: HashMap::new(),
        }
    }

    pub fn get() -> &'static Mutex<AnalyticsHandlerFactory> {
        &SINGLE_INSTANCE
    }

    pub async fn start() {
        // Register all handlers
        // config::CFG.botwaf.analytics
        //     .iter()
        //     .filter(|x| x.kind == SimpleLlmAnalyticsHandler::KIND)
        //     .for_each(|x| {
        //         SimpleLlmAnalyticsHandler::new_and_init(x).await;
        //     });

        // Register all handlers
        for config in &config::CFG.botwaf.analytics {
            if config.kind == SimpleLlmAnalyticsHandler::KIND {
                tracing::info!("Initializing implementation handler: {}", config.name);
                let handler = SimpleLlmAnalyticsHandler::init(config).await;
                if let Err(e) = AnalyticsHandlerFactory::get()
                    .lock()
                    .await
                    .register(config.name.to_owned(), handler.clone())
                    .await
                {
                    tracing::error!("Failed to register LLM analytics handler: {}", e);
                }
                tracing::info!("Registered implementation handler: {}", config.name);
            }
        }

        // Start up all handlers
        for config in &config::CFG.botwaf.analytics {
            match Self::get().lock().await.get_implementation(config.name.to_owned()).await {
                Ok(handler) => {
                    tracing::info!("Starting implementation handler: {}", config.name);
                    handler.start().await;
                }
                Err(_) => {}
            }
        }
    }

    pub async fn register(&mut self, name: String, handler: Arc<dyn IAnalyticsHandler + Send + Sync>) -> Result<(), Error> {
        // Check if the name already exists
        if self.implementations.contains_key(&name) {
            let errmsg = format!("Handler Factory: Name '{}' already exists", name);
            return Err(Error::msg(errmsg));
        }
        self.implementations.insert(name, handler);
        Ok(())
    }

    pub async fn get_implementation(&self, name: String) -> Result<Arc<dyn IAnalyticsHandler + Send + Sync>, Error> {
        if let Some(implementation) = self.implementations.get(&name) {
            Ok(implementation.clone())
        } else {
            let errmsg = format!("Handler Factory: Name '{}' does't exists", name);
            return Err(Error::msg(errmsg));
        }
    }
}

#[derive(Clone)]
pub struct BotWafAccessEvent {
    pub uuid: String,
    pub path: String,
    pub method: String,
    pub headers: String,
    pub body: String,
    pub status_code: i32,
    pub response_headers: String,
    pub response_body: String,
    pub duration: i64,
    pub timestamp: String,
}
