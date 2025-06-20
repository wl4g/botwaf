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

use super::llm_base::ILLMHandler;
use crate::config::config::{self, LlmProperties};
use anyhow::{Ok, Result};
use botwaf_types::modules::llm::knowledge::{KnowledgeCategory, KnowledgeStatus, KnowledgeUploadInfo};
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
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    sync::Arc,
};

/// see:https://github.com/wl4g-ai/langchain-rust/blob/main/examples/conversational_retriever_chain_with_vector_store.rs
pub struct LangchainLLMHandler {
    pgvec_store: Arc<Box<dyn VectorStore>>,
    openai_llm: OpenAI<OpenAIConfig>,
}

impl LangchainLLMHandler {
    pub const NAME: &'static str = "LANGCHAIN";

    #[allow(unused)]
    pub async fn new(config: &LlmProperties) -> Arc<Self> {
        // Create the embedding openai config.
        let llm_config = &config::get_config().services.llm;
        let mut embedding_openai_config =
            OpenAIConfig::new().with_api_base(&config::get_config().services.llm.embedding.api_uri);
        if let Some(api_key) = &config::get_config().services.llm.embedding.api_key {
            // Default used by 'OPENAI_KEY' and 'OPENAI_BASE_URL'.
            // Not require API key to run model by Ollama default.
            embedding_openai_config = embedding_openai_config.with_api_key(api_key);
        }
        if let Some(org_id) = &config::get_config().services.llm.embedding.org_id {
            embedding_openai_config = embedding_openai_config.with_org_id(org_id);
        }
        if let Some(project_id) = &config::get_config().services.llm.embedding.project_id {
            embedding_openai_config = embedding_openai_config.with_org_id(project_id);
        }

        let vecdb_config = &config::get_config().vecdb;
        let pgconn_url = format!(
            "postgresql://{}:{}@{}:{}/{}?schema={}",
            vecdb_config.pg_vector.username,
            vecdb_config.pg_vector.password.to_owned().unwrap_or_default(),
            vecdb_config.pg_vector.host,
            vecdb_config.pg_vector.port,
            vecdb_config.pg_vector.database,
            vecdb_config.pg_vector.schema,
        );
        // Create the knowledge vector store for PG vector.
        let pgvec_store = StoreBuilder::new()
            .embedder(OpenAiEmbedder::new(embedding_openai_config))
            .pre_delete_collection(false)
            .connection_url(pgconn_url.as_str())
            .vector_dimensions(1536)
            .build()
            .await
            .unwrap();

        // Create call LLM config for openai compability.
        let mut call_openai_config = OpenAIConfig::new().with_api_base(&llm_config.generate.api_uri);
        if let Some(api_key) = &llm_config.generate.api_key {
            call_openai_config = call_openai_config.with_api_key(api_key);
        }
        if let Some(org_id) = &llm_config.generate.org_id {
            call_openai_config = call_openai_config.with_org_id(org_id);
        }
        if let Some(project_id) = &llm_config.generate.project_id {
            call_openai_config = call_openai_config.with_org_id(project_id);
        }

        // Create the call LLM client for openai compability.
        let call_opts = CallOptions::new()
            .with_max_tokens(config::get_config().services.llm.generate.max_tokens)
            .with_temperature(config::get_config().services.llm.generate.temperature)
            .with_candidate_count(config::get_config().services.llm.generate.candidate_count)
            // TODO: whether the support configuration of this items?
            .with_functions(Vec::new())
            .with_stop_words(Vec::new())
            .with_top_k(config::get_config().services.llm.generate.top_k)
            .with_top_p(config::get_config().services.llm.generate.top_p)
            // .with_seed(0)
            .with_function_call_behavior(FunctionCallBehavior::Auto);
        let openai_llm = OpenAI::new(call_openai_config)
            .with_model(config::get_config().services.llm.generate.model.to_owned())
            .with_options(call_opts);

        // Create the this updater handler instance.
        Arc::new(Self {
            pgvec_store: Arc::new(Box::new(pgvec_store)),
            openai_llm,
        })
    }
}

#[async_trait::async_trait]
impl ILLMHandler for LangchainLLMHandler {
    async fn init(&self) {}

    async fn embedding(&self, mut info: KnowledgeUploadInfo, file: File) -> Result<KnowledgeUploadInfo, anyhow::Error> {
        info.status = KnowledgeStatus::RECEIVED;

        // TODO: Update to upload table.
        // ...

        info.status = KnowledgeStatus::PERSISTING;
        // TODO: Upload to Object Storage for backup raw file
        // ...

        info.status = KnowledgeStatus::PREPARING;
        // TODO: Update to upload table.
        // ...

        // Parse file into documents
        let reader = BufReader::new(file);
        let mut documents = Vec::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            if let std::result::Result::Ok(content) = line_result {
                if content.trim().is_empty() {
                    continue;
                }

                // Create metadata for sample document.
                let mut metadata = HashMap::new();
                metadata.insert("filename".to_string(), info.name.clone().into());
                metadata.insert("linenum".to_string(), line_num.to_string().into());

                // Addidtion the user-provided labels.
                for (key, value) in &info.labels {
                    metadata.insert(key.clone(), value.clone().into());
                }

                documents.push(Document::new(&content).with_metadata(metadata));
            }
        }

        let store_options = match info.category {
            KnowledgeCategory::NORMAL => {
                VecStoreOptions::new()
                    .with_name_space("NORMAL") // Normal requests (positive category samples)
                    .with_score_threshold(0.5 as f32) // TODO: score threshold
            }
            KnowledgeCategory::MALICIOUS => {
                VecStoreOptions::new()
                    .with_name_space("MALICIOUS") // Maybe attack malicious requests (negative category sample)
                    .with_score_threshold(0.5 as f32) // TODO: score threshold
            }
        };

        info.status = KnowledgeStatus::EMBEDDING;
        // TODO: Update to upload table.
        // ...

        match self.pgvec_store.add_documents(&documents, &store_options).await {
            std::result::Result::Ok(_) => {
                tracing::info!("Embedding success.");
                info.status = KnowledgeStatus::EMBEDDED;
                // TODO: update to upload table.
                // ...
            }
            Err(e) => {
                tracing::error!("Embedding failed: {}", e);
                info.status = KnowledgeStatus::FAILED;
                // TODO: update to upload table.
                // ...
            }
        }

        Ok(info)
    }

    async fn generate(&self, prompt: String) -> Result<String, anyhow::Error> {
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

        // TODO: retriever & generate.
        let prompt= message_formatter![
                    fmt_message!(Message::new_system_message("You are a helpful assistant")),
                    fmt_template!(HumanMessagePromptTemplate::new(template_jinja2!("
Use the following pieces of context to answer the question at the end. If you don't know the answer, just say that you don't know, don't try to make up an answer.

{{context}}

Question:{{question}}

Helpful Answer:
        ", "context","question")))
                ];

        let opts = VecStoreOptions::new()
            .with_name_space("botwaf") // TODO: namespace
            .with_score_threshold(0.3 as f32); // TODO: score threshold

        let retriever = Retriever::new(
            Arc::try_unwrap(self.pgvec_store.to_owned())
                .unwrap_or_else(|_| panic!("Failed to initial retriever with pgvec store.")),
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
        if let std::result::Result::Ok(result) = result {
            println!("Result: {:?}", result);
        }

        todo!()
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[tokio::test]
    // async fn test_llm_vector_store() {
    //     // Attack requests (negative samples)
    //     let attack_samples = vec![
    //         Document::new("192.168.1.1 - - [10/Feb/2024:13:55:36 +0000] \"GET /admin.php?id=1%27%20OR%201=1%20--%20 HTTP/1.1\" 200 2326")
    //             .with_metadata(HashMap::from([
    //                 ("key1".to_string(), "value1".into()),
    //                 ("key2".to_string(), "value2".into()),
    //             ])),
    //         Document::new("192.168.1.2 - - [10/Feb/2024:14:03:21 +0000] \"POST /login.php HTTP/1.1\" 200 1538 \"<script>alert(1)</script>\"")
    //             .with_metadata(HashMap::from([
    //                 ("key1".to_string(), "value1".into()),
    //                 ("key2".to_string(), "value2".into()),
    //             ])),
    //         // More attack samples ...
    //     ];

    //     // Normal requests (positive sample)
    //     let normal_samples =
    //         vec![
    //             Document::new("192.168.1.3 - - [10/Feb/2024:14:07:09 +0000] \"GET /index.php HTTP/1.1\" 200 1538")
    //                 .with_metadata(HashMap::from([
    //                     ("key1".to_string(), "value1".into()),
    //                     ("key2".to_string(), "value2".into()),
    //                 ])),
    //         ];

    //     let documents = vec![
    //         Document::new(format!(
    //             "\nQuestion: {}\nAnswer: {}\n",
    //             "Which is the favorite text editor of luis", "Nvim"
    //         )),
    //         Document::new(format!("\nQuestion: {}\nAnswer: {}\n", "How old is Luis", "24")),
    //         Document::new(format!("\nQuestion: {}\nAnswer: {}\n", "Where do luis live", "Peru")),
    //         Document::new(format!(
    //             "\nQuestion: {}\nAnswer: {}\n",
    //             "Whats his favorite food", "Pan con chicharron"
    //         )),
    //     ];

    //     todo!()
    // }
}
