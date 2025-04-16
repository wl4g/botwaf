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

use crate::{config::config, llm::handler::llm_langchain::LangchainLLMHandler};
use anyhow::Error;
use botwaf_types::knowledge::KnowledgeUploadInfo;
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    fs::File,
    sync::{self, Arc, RwLock},
};

#[async_trait::async_trait]
pub trait ILLMHandler {
    async fn init(&self);
    async fn embedding(&self, mut info: KnowledgeUploadInfo, file: File) -> Result<KnowledgeUploadInfo, anyhow::Error>;
    async fn generate(&self, prompt: String) -> Result<String, anyhow::Error>;
}

lazy_static! {
    static ref SINGLE_INSTANCE: RwLock<LLMManager> = RwLock::new(LLMManager::new());
}

pub struct LLMManager {
    pub implementations: HashMap<String, sync::Arc<dyn ILLMHandler + Send + Sync>>,
}

impl LLMManager {
    fn new() -> Self {
        LLMManager {
            implementations: HashMap::new(),
        }
    }

    pub fn get() -> &'static RwLock<LLMManager> {
        &SINGLE_INSTANCE
    }

    pub async fn init() {
        let config = &config::get_config().services.llm;

        tracing::info!("Initializing implementation langChain LLM ...");
        match Self::get()
            .write() // If acquire fails, then it block until acquired.
            .unwrap() // If acquire fails, then it should panic.
            .register(
                LangchainLLMHandler::NAME.to_owned(),
                LangchainLLMHandler::new(config).await,
            ) {
            Ok(registered) => {
                tracing::info!("Initializing langChain LLM ...");
                let _ = registered.init().await;
            }
            Err(e) => panic!("Failed to register langChain LLM: {}", e),
        }
    }

    fn register<T: ILLMHandler + Send + Sync + 'static>(
        &mut self,
        name: String,
        handler: Arc<T>,
    ) -> Result<Arc<T>, Error> {
        // Check if the name already exists
        if self.implementations.contains_key(&name) {
            tracing::debug!("Already register the LLM handler '{}'", name);
            return Ok(handler);
        }
        self.implementations.insert(name, handler.to_owned());
        Ok(handler)
    }

    pub fn get_implementation(name: String) -> Result<Arc<dyn ILLMHandler + Send + Sync>, Error> {
        // If the read lock is poisoned, the program will panic.
        let this = LLMManager::get().read().unwrap();
        if let Some(implementation) = this.implementations.get(&name) {
            Ok(implementation.to_owned())
        } else {
            let errmsg = format!("Could not obtain registered LLM handler '{}'.", name);
            return Err(Error::msg(errmsg));
        }
    }

    pub fn get_default_implementation() -> Arc<dyn ILLMHandler + Send + Sync> {
        Self::get_implementation(LangchainLLMHandler::NAME.to_owned()).expect("Failed to get default LLM handler")
    }
}
