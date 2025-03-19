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

use super::verifier_execution::SimpleExecuteBasedVerifier;

#[async_trait]
pub trait IBotwafVerifier: Send + Sync {
    async fn init(&self);
}

lazy_static! {
    static ref SINGLE_INSTANCE: Mutex<BotwafVerifierManager> = Mutex::new(BotwafVerifierManager::new());
}

pub struct BotwafVerifierManager {
    pub implementations: HashMap<String, Arc<dyn IBotwafVerifier + Send + Sync>>,
}

impl BotwafVerifierManager {
    fn new() -> Self {
        BotwafVerifierManager {
            implementations: HashMap::new(),
        }
    }

    pub fn get() -> &'static Mutex<BotwafVerifierManager> {
        &SINGLE_INSTANCE
    }

    pub async fn init() {
        tracing::info!("Register All Botwaf verifiers ...");
        for config in &config::CFG.botwaf.verifiers {
            if config.kind == SimpleExecuteBasedVerifier::KIND {
                tracing::info!("Initializing implementation Botwaf verifier: {}", config.name);
                let handler = SimpleExecuteBasedVerifier::new(config).await;
                if let Err(e) = BotwafVerifierManager::get()
                    .lock()
                    .await
                    .register(config.name.to_owned(), handler.to_owned())
                {
                    tracing::error!("Failed to register Botwaf verifier: {}", e);
                }
                tracing::info!("Registered implementation Botwaf verifier: {}", config.name);
            }
        }

        tracing::info!("Initializing All Botwaf verifiers ...");
        for config in &config::CFG.botwaf.verifiers {
            match Self::get()
                .lock()
                .await
                .get_implementation(config.name.to_owned())
                .await
            {
                Ok(handler) => {
                    tracing::info!("Initializing implementation Botwaf verifier: {}", config.name);
                    handler.init().await;
                }
                Err(_) => {}
            }
        }
    }

    fn register(&mut self, name: String, handler: Arc<dyn IBotwafVerifier + Send + Sync>) -> Result<(), Error> {
        // Check if the name already exists
        if self.implementations.contains_key(&name) {
            let errmsg = format!("Verifier Factory: Name '{}' already exists", name);
            return Err(Error::msg(errmsg));
        }
        self.implementations.insert(name, handler);
        Ok(())
    }

    pub async fn get_implementation(&self, name: String) -> Result<Arc<dyn IBotwafVerifier + Send + Sync>, Error> {
        if let Some(implementation) = self.implementations.get(&name) {
            Ok(implementation.clone())
        } else {
            let errmsg = format!("Verifier Factory: Name '{}' does't exists", name);
            return Err(Error::msg(errmsg));
        }
    }
}
