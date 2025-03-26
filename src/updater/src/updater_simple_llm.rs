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

// use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
use async_trait::async_trait;

use super::updater_base::{BotwafAccessEvent, IBotwafUpdater};
use botwaf_server::config::config::{self, UpdaterProperties};
use langchain_rust::{
    chain::{Chain, ConversationalRetrieverChainBuilder},
    embedding::openai::OpenAiEmbedder,
    fmt_message, fmt_template,
    language_models::options::CallOptions,
    llm::{OpenAI, OpenAIConfig},
    memory::WindowBufferMemory,
    message_formatter,
    prompt::HumanMessagePromptTemplate,
    prompt_args,
    schemas::{Document, FunctionCallBehavior, Message},
    template_jinja2,
    vectorstore::{pgvector::StoreBuilder, Retriever, VecStoreOptions, VectorStore},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

#[derive(Clone)]
pub struct SimpleLLMUpdater {
    config: UpdaterProperties,
    scheduler: Arc<JobScheduler>,
    pgvec_store: Arc<Box<dyn VectorStore>>,
    openai_llm: OpenAI<OpenAIConfig>,
}

impl SimpleLLMUpdater {
    pub const KIND: &'static str = "SIMPLE_LLM";

    pub async fn new(config: &UpdaterProperties) -> Arc<Self> {
        // Create the embedding openai config.
        let mut embed_openai_config =
            OpenAIConfig::new().with_api_base(&config::get_config().botwaf.llm.embedding.api_uri);
        if let Some(api_key) = &config::get_config().botwaf.llm.embedding.api_key {
            // Default used by 'OPENAI_KEY' and 'OPENAI_BASE_URL'.
            // Not require API key to run model by Ollama default.
            embed_openai_config = embed_openai_config.with_api_key(api_key);
        }
        if let Some(org_id) = &config::get_config().botwaf.llm.embedding.org_id {
            embed_openai_config = embed_openai_config.with_org_id(org_id);
        }
        if let Some(project_id) = &config::get_config().botwaf.llm.embedding.project_id {
            embed_openai_config = embed_openai_config.with_org_id(project_id);
        }

        // Create the knowledge vector store for PG vector.
        let pgvec_store = StoreBuilder::new()
            .embedder(OpenAiEmbedder::new(embed_openai_config))
            .pre_delete_collection(true)
            // TODO:
            .connection_url("postgresql://postgres:postgres@localhost:5432/postgres")
            .vector_dimensions(1536)
            .build()
            .await
            .unwrap();

        // Create call LLM config for openai compability.
        let mut call_openai_config =
            OpenAIConfig::new().with_api_base(&config::get_config().botwaf.llm.generate.api_uri);
        if let Some(api_key) = &config::get_config().botwaf.llm.generate.api_key {
            call_openai_config = call_openai_config.with_api_key(api_key);
        }
        if let Some(org_id) = &config::get_config().botwaf.llm.generate.org_id {
            call_openai_config = call_openai_config.with_org_id(org_id);
        }
        if let Some(project_id) = &config::get_config().botwaf.llm.generate.project_id {
            call_openai_config = call_openai_config.with_org_id(project_id);
        }

        // Create the call LLM client for openai compability.
        // TODO: Should be used configuration.
        let opts = CallOptions::new()
            .with_max_tokens(65535)
            .with_temperature(0.1) // TODO: Should be as low as possible?
            .with_candidate_count(3) // TODO:
            // .with_functions(Vec::new()) // TODO:
            // .with_stop_words(Vec::new()) // TODO:
            // .with_top_k(3) // TODO:
            // .with_top_p(0.5 as f32) // TODO:
            // .with_seed(0) // TODO:
            .with_function_call_behavior(FunctionCallBehavior::Auto);
        let openai_llm = OpenAI::new(call_openai_config).with_model("model").with_options(opts);

        // Create the this updater handler instance.
        Arc::new(Self {
            config: config.to_owned(),
            scheduler: Arc::new(JobScheduler::new_with_channel_size(config.channel_size).await.unwrap()),
            pgvec_store: Arc::new(Box::new(pgvec_store)),
            openai_llm,
        })
    }

    pub(super) async fn update(&self) {
        tracing::info!("Simple LLM updating ...");

        // Native OpenAI to completions.
        // let messages = vec![ChatCompletionMessage {
        //     role: ChatCompletionMessageRole::System,
        //     content: Some("You are a helpful assistant.".to_string()),
        //     name: None,
        //     function_call: None,
        //     tool_call_id: None,
        //     tool_calls: None,
        // }];
        // let embedding_result = ChatCompletion::builder(&config::get_config().botwaf.llm.embedding.model, messages.clone())
        //     .credentials(self.embedding_openai_config.as_ref().to_owned())
        //     .create()
        //     .await;
        // match embedding_result {
        //     Ok(ret) => {
        //         let msg = ret.choices.first().unwrap().message.clone();
        //         // Assistant: Sure! Here's a random crab fact: ...
        //         tracing::info!("{:#?}: {}", msg.role, msg.content.unwrap().trim());
        //         // send to LLM to analyze
        //         // update the ModSecurity rule to state according LLM analysis result
        //     }
        //     Err(e) => {
        //         tracing::error!("Failed to LLM embedding: {:?}", e);
        //         return;
        //     }
        // }

        // TODO
        // for event in self.fetch_events(0, 100).await {
        //     tracing::info!("Scanning access event: {}", event.uuid);
        // }

        // TODO: embedding into vector store
        // for testing only, after should be add to pretrain samples upload api.

        // Attack requests (negative samples)
        let attack_samples = vec![
            Document::new("192.168.1.1 - - [10/Feb/2024:13:55:36 +0000] \"GET /admin.php?id=1%27%20OR%201=1%20--%20 HTTP/1.1\" 200 2326")
                .with_metadata(HashMap::from([
                    ("key1".to_string(), "value1".into()),
                    ("key2".to_string(), "value2".into()),
                ])),
            Document::new("192.168.1.2 - - [10/Feb/2024:14:03:21 +0000] \"POST /login.php HTTP/1.1\" 200 1538 \"<script>alert(1)</script>\"")
                .with_metadata(HashMap::from([
                    ("key1".to_string(), "value1".into()),
                    ("key2".to_string(), "value2".into()),
                ])),
            // More attack samples ...
        ];

        // Normal requests (positive sample)
        let normal_samples =
            vec![
                Document::new("192.168.1.3 - - [10/Feb/2024:14:07:09 +0000] \"GET /index.php HTTP/1.1\" 200 1538")
                    .with_metadata(HashMap::from([
                        ("key1".to_string(), "value1".into()),
                        ("key2".to_string(), "value2".into()),
                    ])),
            ];

        // 存储到不同的命名空间
        let attack_opts = VecStoreOptions::new()
            .with_name_space("malicious")
            .with_score_threshold(0.5 as f32); // TODO: score threshold;
        let normal_opts = VecStoreOptions::new()
            .with_name_space("normal")
            .with_score_threshold(0.5 as f32); // TODO: score threshold

        let _ = self.pgvec_store.add_documents(&attack_samples, &attack_opts).await;
        let _ = self.pgvec_store.add_documents(&normal_samples, &normal_opts).await;

        let docs = vec![
            Document::new(format!(
                "\nQuestion: {}\nAnswer: {}\n",
                "Which is the favorite text editor of luis", "Nvim"
            )),
            Document::new(format!("\nQuestion: {}\nAnswer: {}\n", "How old is Luis", "24")),
            Document::new(format!("\nQuestion: {}\nAnswer: {}\n", "Where do luis live", "Peru")),
            Document::new(format!(
                "\nQuestion: {}\nAnswer: {}\n",
                "Whats his favorite food", "Pan con chicharron"
            )),
        ];

        let opts = VecStoreOptions::new()
            .with_name_space("botwaf") // TODO: namespace
            .with_score_threshold(0.3 as f32); // TODO: score threshold
        let _ = self.pgvec_store.add_documents(&docs, &opts);

        // TODO: retriever & generate
        let prompt= message_formatter![
                    fmt_message!(Message::new_system_message("You are a helpful assistant")),
                    fmt_template!(HumanMessagePromptTemplate::new(template_jinja2!("
Use the following pieces of context to answer the question at the end. If you don't know the answer, just say that you don't know, don't try to make up an answer.

{{context}}

Question:{{question}}

Helpful Answer:
        ", "context","question")))
                ];

        let retriever = Retriever::new(
            Arc::try_unwrap(self.pgvec_store.to_owned()).unwrap_or_else(|_| panic!("Failed to unwrap pgvec store.")),
            1024,
        )
        .with_options(opts);
        let chain = ConversationalRetrieverChainBuilder::new()
            .llm(self.openai_llm.to_owned())
            .rephrase_question(true)
            .return_source_documents(true)
            .memory(WindowBufferMemory::new(100).into())
            .retriever(retriever)
            //If you want to use the default prompt remove the .prompt()
            //Keep in mind if you want to change the prompt; this chain need the {{context}} variable
            .prompt(prompt)
            .build()
            .expect("Error building ConversationalChain");

        let input_variables = prompt_args! {
            "question" => "Hi",
            "input" => "Who is the writer of 20,000 Leagues Under the Sea, and what is my name?",
            "history" => vec![
                Message::new_human_message("My name is: Luis"),
                Message::new_ai_message("Hi Luis"),
            ],
        };

        let result = chain.invoke(input_variables).await;
        if let Ok(result) = result {
            println!("Result: {:?}", result);
        }
    }

    #[allow(unused)]
    async fn fetch_events(&self, page_index: i64, page_size: i64) -> Vec<BotwafAccessEvent> {
        unimplemented!()
    }
}

#[async_trait]
impl IBotwafUpdater for SimpleLLMUpdater {
    // start async thread job to re-scaning near real-time recorded access events.
    async fn init(&self) {
        let this = self.clone();

        // Pre-check the cron expression is valid.
        let cron = match Job::new_async(self.config.cron.as_str(), |_uuid, _lock| Box::pin(async {})) {
            Ok(_) => self.config.cron.as_str(),
            Err(e) => {
                tracing::warn!(
                    "Invalid cron expression '{}': {}. Using default '0/30 * * * * *'",
                    self.config.cron,
                    e
                );
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
