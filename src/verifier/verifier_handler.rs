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

use crate::config::config;

use super::verifier_handler_execution::SimpleExecuteBasedHandler;

#[async_trait]
pub trait IVerifierHandler: Send + Sync {
    async fn start(&self);
}

lazy_static! {
    static ref SINGLE_INSTANCE: Mutex<VerifierHandlerManager> = Mutex::new(VerifierHandlerManager::new());
}

pub struct VerifierHandlerManager {
    pub implementations: HashMap<String, Arc<dyn IVerifierHandler + Send + Sync>>,
}

impl VerifierHandlerManager {
    fn new() -> Self {
        VerifierHandlerManager {
            implementations: HashMap::new(),
        }
    }

    pub fn get() -> &'static Mutex<VerifierHandlerManager> {
        &SINGLE_INSTANCE
    }

    pub async fn start() {
        tracing::info!("Initializing to verifier handlers ...");

        for config in &config::CFG.botwaf.verifiers {
            if config.kind == SimpleExecuteBasedHandler::KIND {
                tracing::info!("Initializing implementation verifier handler: {}", config.name);
                let handler = SimpleExecuteBasedHandler::init(config).await;
                if let Err(e) = VerifierHandlerManager::get()
                    .lock()
                    .await
                    .register(config.name.to_owned(), handler.clone())
                    .await
                {
                    tracing::error!("Failed to register verifier handler: {}", e);
                }
                tracing::info!("Registered implementation verifier handler: {}", config.name);
            }
        }

        // Start up all handlers
        for config in &config::CFG.botwaf.verifiers {
            match Self::get().lock().await.get_implementation(config.name.to_owned()).await {
                Ok(handler) => {
                    tracing::info!("Starting implementation verifier handler: {}", config.name);
                    handler.start().await;
                }
                Err(_) => {}
            }
        }
    }

    pub async fn register(&mut self, name: String, handler: Arc<dyn IVerifierHandler + Send + Sync>) -> Result<(), Error> {
        // Check if the name already exists
        if self.implementations.contains_key(&name) {
            let errmsg = format!("Verifier Handler Factory: Name '{}' already exists", name);
            return Err(Error::msg(errmsg));
        }
        self.implementations.insert(name, handler);
        Ok(())
    }

    pub async fn get_implementation(&self, name: String) -> Result<Arc<dyn IVerifierHandler + Send + Sync>, Error> {
        if let Some(implementation) = self.implementations.get(&name) {
            Ok(implementation.clone())
        } else {
            let errmsg = format!("Handler Factory: Name '{}' does't exists", name);
            return Err(Error::msg(errmsg));
        }
    }
}
