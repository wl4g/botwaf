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

use crate::config::config::{self, UpdaterProperties};
use async_trait::async_trait;
use openai::{
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
    Credentials,
};
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

use super::updater_handler::{BotWafAccessEvent, IUpdaterHandler};

#[derive(Clone)]
pub struct SimpleLLMUpdaterHandler {
    config: UpdaterProperties,
    scheduler: Arc<JobScheduler>,
    embedding_credentials: Arc<Credentials>,
    generate_credentials: Arc<Credentials>,
}

impl SimpleLLMUpdaterHandler {
    pub const KIND: &'static str = "SIMPLE_LLM";

    pub async fn init(config: &UpdaterProperties) -> Arc<Self> {
        Arc::new(Self {
            config: config.to_owned(),
            scheduler: Arc::new(JobScheduler::new_with_channel_size(config.channel_size).await.unwrap()),
            // Default used by 'OPENAI_KEY' and 'OPENAI_BASE_URL'.
            // Not require API key to run model by Ollama default.
            embedding_credentials: Arc::new(Credentials::new(
                &config::CFG.botwaf.llm.embedding.api_key.to_owned().unwrap_or_default(),
                &config::CFG.botwaf.llm.embedding.api_uri,
            )),
            generate_credentials: Arc::new(Credentials::new(
                &config::CFG.botwaf.llm.generate.api_key.to_owned().unwrap_or_default(),
                &config::CFG.botwaf.llm.generate.api_uri,
            )),
        })
    }

    pub(super) async fn update(&self) {
        tracing::info!("Simple LLM updating ...");

        // TODO
        // for event in self.fetch_events(0, 100).await {
        //     tracing::info!("Scanning access event: {}", event.uuid);
        // }

        let messages = vec![ChatCompletionMessage {
            role: ChatCompletionMessageRole::System,
            content: Some("You are a helpful assistant.".to_string()),
            name: None,
            function_call: None,
            tool_call_id: None,
            tool_calls: None,
        }];
        let embedding_result = ChatCompletion::builder(&config::CFG.botwaf.llm.embedding.model, messages.clone())
            .credentials(self.embedding_credentials.as_ref().to_owned())
            .create()
            .await;
        match embedding_result {
            Ok(ret) => {
                let msg = ret.choices.first().unwrap().message.clone();
                // Assistant: Sure! Here's a random crab fact: ...
                tracing::info!("{:#?}: {}", msg.role, msg.content.unwrap().trim());
                // send to LLM to analyze
                // update the ModSecurity rule to state according LLM analysis result
            }
            Err(e) => {
                tracing::error!("Failed to LLM embedding: {:?}", e);
                return;
            }
        }
    }

    async fn fetch_events(&self, page_index: i64, page_size: i64) -> Vec<BotWafAccessEvent> {
        unimplemented!()
    }
}

#[async_trait]
impl IUpdaterHandler for SimpleLLMUpdaterHandler {
    // start async thread job to re-scaning near real-time recorded access events.
    async fn start(&self) {
        let this = self.clone();

        // Pre-check the cron expression is valid.
        let cron = match Job::new_async(self.config.cron.as_str(), |_uuid, _lock| Box::pin(async {})) {
            Ok(_) => self.config.cron.as_str(),
            Err(e) => {
                tracing::warn!("Invalid cron expression '{}': {}. Using default '0/30 * * * * *'", self.config.cron, e);
                "0/30 * * * * *" // every half minute
            }
        };

        tracing::info!("Starting Analytics handler with cron '{}'", cron);
        let job = Job::new_async(cron, move |_uuid, _lock| {
            let that = this.clone();
            Box::pin(async move {
                tracing::info!("{:?} Hi I ran", chrono::Utc::now());
                that.update().await;
            })
        })
        .unwrap();

        self.scheduler.add(job).await.unwrap();
        self.scheduler.start().await.unwrap();

        tracing::info!("Started Simple LLM Analytics handler.");
        // Notice: It's will keep the program running
        // tokio::signal::ctrl_c().await.unwrap();
    }
}

#[cfg(test)]
mod tests {
    // use std::env;
    // use crate::config::config::{ AppConfigProperties, LlmProperties };
    // use super::*;

    // #[tokio::test]
    // async fn test_analyze_with_qwen() {
    //     let mut config = AppConfigProperties::default();

    //     let mut analyze_config = &AnalyticsProperties::default();
    //     analyze_config.kind = SimpleLlmAnalyticsHandler::KIND.to_owned();
    //     analyze_config.name = "defaultAnalyze".to_string();
    //     analyze_config.cron = "0/10 * * * * *".to_string();
    //     config.botwaf.analytics.push(analyze_config);

    //     let mut llm_config = LlmProperties::default();
    //     //llm_config.api_url = "https://api.openai.com/v1/chat/completions".to_string();
    //     llm_config.api_url = "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string();
    //     llm_config.api_key = env::var("TEST_OPENAI_KEY").ok().unwrap();
    //     //llm_config.model = "gpt-3.5-turbo".to_string();
    //     llm_config.model = "qwen-plus".to_string();
    //     config.botwaf.llm = llm_config;

    //     let handler = SimpleLlmAnalyticsHandler::init(analyze_config).await;
    //     handler.analyze().await;
    // }
}
