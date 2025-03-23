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

use std::{env, ops::Deref, sync::Arc};

use crate::logging::logging::LogMode;
use config::Config;
use dotenv::dotenv;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use validator::Validate;

// Global program information.
pub const GIT_VERSION: &str = env!("GIT_VERSION");
pub const GIT_COMMIT_HASH: &str = env!("GIT_COMMIT_HASH");
pub const GIT_BUILD_DATE: &str = env!("GIT_BUILD_DATE");

lazy_static! {
    pub static ref VERSION: String = format!(
        "GitVersion: {}, GitHash: {}, GitBuildDate: {}",
        env!("GIT_VERSION"),
        env!("GIT_COMMIT_HASH"),
        env!("GIT_BUILD_DATE")
    );
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfigProperties {
    #[serde(rename = "service-name")]
    #[validate(length(min = 1, max = 32))]
    pub service_name: String,
    #[serde(default = "ServerProperties::default")]
    pub server: ServerProperties,
    #[serde(default = "SwaggerProperties::default")]
    pub swagger: SwaggerProperties,
    #[serde(default = "LoggingProperties::default")]
    pub logging: LoggingProperties,
    #[serde(default = "CacheProperties::default")]
    pub cache: CacheProperties,
    #[serde(default = "DatabaseProperties::default")]
    pub database: DatabaseProperties,
    #[serde(default = "BotwafProperties::default")]
    pub botwaf: BotwafProperties,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerProperties {
    #[serde(rename = "host")]
    pub host: String,
    #[serde(rename = "port")]
    pub port: u16,
    #[serde(rename = "context-path")]
    pub context_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SwaggerProperties {
    pub enabled: bool,
    // pub title: String,
    // pub description: String,
    // pub version: String,
    // pub license_name: String,
    // pub license_url: String,
    // pub contact_name: String,
    // pub contact_email: String,
    // pub contact_url: String,
    // pub terms_of_service: String,
    // //pub security_definitions: vec![],
    pub swagger_ui_path: String,
    pub swagger_openapi_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingProperties {
    pub mode: LogMode,
    pub level: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheProperties {
    pub provider: CacheProvider,
    pub memory: MemoryProperties,
    pub redis: RedisProperties,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CacheProvider {
    MEMORY,
    REDIS,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryProperties {
    #[serde(rename = "initial-capacity")]
    pub initial_capacity: Option<u32>,
    #[serde(rename = "max-capacity")]
    pub max_capacity: Option<u64>,
    pub ttl: Option<u64>,
    #[serde(rename = "eviction-policy")]
    pub eviction_policy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedisProperties {
    pub nodes: Vec<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(rename = "connection-timeout")]
    pub connection_timeout: Option<u64>,
    #[serde(rename = "response-timeout")]
    pub response_timeout: Option<u64>,
    pub retries: Option<u32>,
    #[serde(rename = "max-retry-wait")]
    pub max_retry_wait: Option<u64>,
    #[serde(rename = "min-retry-wait")]
    pub min_retry_wait: Option<u64>,
    #[serde(rename = "read-from-replicas")]
    pub read_from_replicas: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseProperties {
    #[serde(rename = "systemdb", default = "SystemDBProperties::default")]
    pub systemdb: SystemDBProperties,
    #[serde(rename = "vectorpg", default = "PgVectorProperties::default")]
    pub vectordb: PgVectorProperties,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostgresProperties {
    #[serde(rename = "host")]
    pub host: String,
    #[serde(rename = "port")]
    pub port: u16,
    #[serde(rename = "database")]
    pub database: String,
    #[serde(rename = "schema")]
    pub schema: String,
    #[serde(rename = "username")]
    pub username: String,
    #[serde(rename = "password")]
    pub password: Option<String>,
    #[serde(rename = "min-connections")]
    pub min_connections: Option<u32>,
    #[serde(rename = "max-connections")]
    pub max_connections: Option<u32>,
    #[serde(rename = "use-ssl")]
    pub use_ssl: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemDBProperties {
    #[serde(flatten)]
    pub inner: PostgresProperties,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PgVectorProperties {
    #[serde(flatten)]
    pub inner: PostgresProperties,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BotwafProperties {
    // Notice: Nginx support status code range: 300-599.
    #[serde(rename = "blocked-status-code")]
    pub blocked_status_code: Option<u16>,
    #[serde(rename = "blocked-header-name")]
    pub blocked_header_name: String,
    #[serde(rename = "allow-addition-modsec-info")]
    pub allow_addition_modsec_info: bool,
    #[serde(rename = "static-rules")]
    pub static_rules: Vec<StaticRule>,
    #[serde(rename = "updaters")]
    pub updaters: Vec<UpdaterProperties>,
    #[serde(rename = "verifiers")]
    pub verifiers: Vec<VerifierProperties>,
    #[serde(rename = "llm", default = "LlmProperties::default")]
    pub llm: LlmProperties,
    #[serde(rename = "forward", default = "ForwardProperties::default")]
    pub forward: ForwardProperties,
}

/// ModSec rules updater based LLM, and similar design as k8s multi specification controller implementation.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdaterProperties {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "kind")]
    pub kind: String,
    #[serde(rename = "enabled")]
    pub enabled: bool,
    #[serde(rename = "cron")]
    pub cron: String,
    #[serde(rename = "channel-size")]
    pub channel_size: usize,
}

/// ModSec rules generated by LLM to verifier, and similar design as k8s multi specification scheduler implementation.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VerifierProperties {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "kind")]
    pub kind: String,
    #[serde(rename = "enabled")]
    pub enabled: bool,
    #[serde(rename = "cron")]
    pub cron: String,
    #[serde(rename = "channel-size")]
    pub channel_size: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LlmProperties {
    // see:https://platform.openai.com/docs/guides/completions
    // see:https://github.com/ollama/ollama/blob/main/docs/api.md#generate-a-completion
    // see:https://help.aliyun.com/zh/model-studio/getting-started/what-is-model-studio#16693d2e3fmir
    #[serde(rename = "embedding")]
    pub embedding: EmbeddingLLMProperties,
    #[serde(rename = "generate")]
    pub generate: GenerateLLMProperties,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbeddingLLMProperties {
    #[serde(rename = "api-uri")]
    pub api_uri: String,
    #[serde(rename = "api-key")]
    pub api_key: Option<String>,
    #[serde(rename = "org-id")]
    pub org_id: Option<String>,
    #[serde(rename = "project-id")]
    pub project_id: Option<String>,
    #[serde(rename = "model")]
    pub model: String,
    #[serde(rename = "pre-delete-collection")]
    pub pre_delete_collection: bool,
    #[serde(rename = "vector-dimensions")]
    pub vector_dimensions: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenerateLLMProperties {
    #[serde(rename = "api-uri")]
    pub api_uri: String,
    #[serde(rename = "api-key")]
    pub api_key: Option<String>,
    #[serde(rename = "org-id")]
    pub org_id: Option<String>,
    #[serde(rename = "project-id")]
    pub project_id: Option<String>,
    #[serde(rename = "model")]
    pub model: String,
    #[serde(rename = "max-tokens")]
    pub max_tokens: u32,
    #[serde(rename = "temperature")]
    pub temperature: f32,
    #[serde(rename = "candidate-count")]
    pub candidate_count: usize,
    #[serde(rename = "top-k")]
    pub top_k: usize,
    #[serde(rename = "top-p")]
    pub top_p: f32,
    #[serde(rename = "system-prompt")]
    pub system_prompt: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForwardProperties {
    #[serde(rename = "max-body-bytes")]
    pub max_body_bytes: usize,
    #[serde(rename = "http-proxy")]
    pub http_proxy: Option<String>,
    #[serde(rename = "connect-timeout")]
    pub connect_timeout: u64,
    #[serde(rename = "read-timeout")]
    pub read_timeout: u64,
    #[serde(rename = "total-timeout")]
    pub total_timeout: u64,
    #[serde(rename = "verbose")]
    pub verbose: bool,
    // Downstream proxy server additional upstream destination header.
    #[serde(rename = "upstream-destination-header-name")]
    pub upstream_destination_header_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StaticRule {
    pub name: String,
    pub kind: String, // Notice: Currently only support "RAW"
    pub severity: String,
    pub desc: String,
    pub value: String,
}

impl AppConfigProperties {
    pub fn default() -> AppConfigProperties {
        AppConfigProperties {
            service_name: String::from("botwaf"),
            server: ServerProperties::default(),
            swagger: SwaggerProperties::default(),
            logging: LoggingProperties::default(),
            cache: CacheProperties::default(),
            database: DatabaseProperties::default(),
            botwaf: BotwafProperties::default(),
        }
    }
}

impl Default for ServerProperties {
    fn default() -> Self {
        ServerProperties {
            host: String::from("127.0.0.1"),
            port: 9999,
            context_path: None,
        }
    }
}

impl Default for SwaggerProperties {
    fn default() -> Self {
        SwaggerProperties {
            enabled: true,
            // title: "My Webnote API Server".to_string(),
            // description: "The My Webnote API Server".to_string(),
            // version: "1.0.0".to_string(),
            // license_name: "Apache 2.0".to_string(),
            // license_url: "https://www.apache.org/licenses/LICENSE-2.0".to_string(),
            // contact_name: "MyWebnote API".to_string(),
            // contact_email: "jameswong1376@gmail.com".to_string(),
            // contact_url: "https://github.com/wl4g/my-webnote".to_string(),
            // terms_of_service: "api/terms-of-service".to_string(),
            // //security_definitions: vec![],
            swagger_ui_path: "/swagger-ui".to_string(),
            swagger_openapi_url: "/api-docs/openapi.json".to_string(),
        }
    }
}

impl Default for LoggingProperties {
    fn default() -> Self {
        LoggingProperties {
            mode: LogMode::JSON,
            level: "info".to_string(),
        }
    }
}

impl Default for CacheProperties {
    fn default() -> Self {
        CacheProperties {
            provider: CacheProvider::MEMORY,
            memory: MemoryProperties::default(),
            redis: RedisProperties::default(),
        }
    }
}

impl Default for MemoryProperties {
    fn default() -> Self {
        MemoryProperties {
            initial_capacity: Some(32),
            max_capacity: Some(65535),
            ttl: Some(3600),
            eviction_policy: Some("lru".to_string()),
        }
    }
}

impl Default for RedisProperties {
    fn default() -> Self {
        RedisProperties {
            nodes: vec!["redis://127.0.0.1:6379".to_string()],
            username: None,
            password: None,
            connection_timeout: Some(3000),
            response_timeout: Some(6000),
            retries: Some(1),
            max_retry_wait: Some(65536),
            min_retry_wait: Some(1280),
            read_from_replicas: Some(false),
        }
    }
}

impl Default for DatabaseProperties {
    fn default() -> Self {
        DatabaseProperties {
            systemdb: SystemDBProperties::default(),
            vectordb: PgVectorProperties::default(),
        }
    }
}

impl Default for PostgresProperties {
    fn default() -> Self {
        PostgresProperties {
            host: String::from("127.0.0.1"),
            port: 5432,
            database: String::from("botwaf"),
            schema: String::from("botwaf"),
            username: String::from("postgres"),
            password: None,
            min_connections: Some(1),
            max_connections: Some(10),
            use_ssl: false,
        }
    }
}

impl Default for SystemDBProperties {
    fn default() -> Self {
        SystemDBProperties {
            inner: PostgresProperties::default(),
        }
    }
}

impl Deref for SystemDBProperties {
    type Target = PostgresProperties;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Default for PgVectorProperties {
    fn default() -> Self {
        PgVectorProperties {
            inner: PostgresProperties::default(),
        }
    }
}

impl Deref for PgVectorProperties {
    type Target = PostgresProperties;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Default for BotwafProperties {
    fn default() -> Self {
        BotwafProperties {
            blocked_status_code: None,
            blocked_header_name: String::from("X-BotWaf-Blocked"),
            allow_addition_modsec_info: true,
            static_rules: vec![],
            llm: LlmProperties::default(),
            updaters: Vec::new(),
            verifiers: Vec::new(),
            forward: ForwardProperties::default(),
        }
    }
}

impl Default for UpdaterProperties {
    fn default() -> Self {
        UpdaterProperties {
            name: String::from("default"),
            kind: String::from("SIMPLE_LLM"),
            enabled: true,
            cron: String::from("0/30 * * * * * *"), // Every half minute
            channel_size: 200,
        }
    }
}

impl Default for VerifierProperties {
    fn default() -> Self {
        VerifierProperties {
            name: String::from("default"),
            kind: String::from("SIMPLE_EXECUTE"),
            enabled: true,
            cron: String::from("0/30 * * * * * *"), // Every half minute
            channel_size: 200,
        }
    }
}

impl Default for LlmProperties {
    fn default() -> Self {
        LlmProperties {
            embedding: EmbeddingLLMProperties::default(),
            generate: GenerateLLMProperties::default(),
        }
    }
}

impl Default for EmbeddingLLMProperties {
    fn default() -> Self {
        EmbeddingLLMProperties {
            api_uri: String::from("https://dashscope.aliyuncs.com/compatible-mode/v1"),
            api_key: None,
            org_id: None,
            project_id: None,
            model: String::from("bge-m3:latest"),
            pre_delete_collection: false,
            vector_dimensions: 1536,
        }
    }
}

impl Default for GenerateLLMProperties {
    fn default() -> Self {
        GenerateLLMProperties {
            api_uri: String::from("https://dashscope.aliyuncs.com/compatible-mode/v1"),
            api_key: None,
            org_id: None,
            project_id: None,
            model: String::from("qwen-plus"),
            max_tokens: 65535,
            candidate_count: 1,
            temperature: 0.1,
            top_k: 1,
            top_p: 1.0,
            system_prompt: String::from(
                "You are a security expert.\n\
                 You are given a list of rules and a request.\n\
                 You must determine if the request is safe or not.\n\
                 If the request is safe, you must return \"safe\".\n\
                 If the request is not safe, you must return \"unsafe\" and provide a reason.\n\
                 You must also provide a list of rules that were used to determine the result.\n\
                 You must also provide a list of rules that were not used to determine the result.",
            ),
        }
    }
}

impl Default for ForwardProperties {
    fn default() -> Self {
        ForwardProperties {
            max_body_bytes: 65535,
            http_proxy: None,
            connect_timeout: 5,
            read_timeout: 5,
            total_timeout: 10,
            verbose: false,
            upstream_destination_header_name: String::from("X-Upstream-Destination"),
        }
    }
}

fn init() -> Arc<AppConfigProperties> {
    dotenv().ok(); // Notice: Must be called before parse from environment file (.env).

    let config = Arc::new(
        env::var("BOTWAF_CFG_PATH")
            .map(|path| {
                Config::builder()
                    .add_source(config::File::with_name(path.as_str()))
                    .add_source(
                        // Extrat candidate from env refer to: https://github.com/rust-cli/config-rs/blob/v0.15.9/src/env.rs#L290
                        // Set up into hierarchy struct attibutes refer to:https://github.com/rust-cli/config-rs/blob/v0.15.9/src/source.rs#L24
                        config::Environment::with_prefix("BOTWAF")
                            // Notice: Use double "_" to distinguish between different hierarchy struct or attribute alies at the same level.
                            .separator("__")
                            .convert_case(config::Case::Cobol)
                            .keep_prefix(true), // Remove the prefix when matching.
                    )
                    .build()
                    .unwrap_or_else(|err| panic!("Error parsing config: {}", err))
                    .try_deserialize::<AppConfigProperties>()
                    .unwrap_or_else(|err| panic!("Error deserialize config: {}", err))
            })
            .unwrap_or(AppConfigProperties::default()),
    );
    if env::var("BOTWAF_CFG_VERBOSE").is_ok() || env::var("VERBOSE").is_ok() {
        println!("If you don't want to print the loaded configuration details, you can disable it by set up BOTWAF_CFG_VERBOSE=false.");
        println!(
            "Loaded the config details: {}",
            serde_json::to_string(config.as_ref()).unwrap()
        );
    }

    return config;
}

lazy_static! {
    pub static ref CFG: Arc<AppConfigProperties> = init();
}
