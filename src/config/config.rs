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

use std::{env, sync::Arc};

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
    #[serde(default = "LoggingProperties::default")]
    pub logging: LoggingProperties,
    #[serde(default = "CacheProperties::default")]
    pub cache: CacheProperties,
    #[serde(default = "BotwafProperties::default")]
    pub botwaf: BotwafProperties,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerProperties {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingProperties {
    pub mode: LogMode,
    pub level: String,
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
    #[serde(rename = "analytics")]
    pub analytics: Vec<AnalyticsProperties>,
    #[serde(rename = "llm", default = "LlmProperties::default")]
    pub llm: LlmProperties,
    #[serde(rename = "forward", default = "ForwardProperties::default")]
    pub forward: ForwardProperties,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheProperties {
    pub provider: CacheProvider,
    pub memory: MemoryProperties,
    pub redis: RedisProperties,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CacheProvider {
    Memory,
    Redis,
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
pub struct StaticRule {
    pub name: String,
    pub kind: String, // Notice: Currently only support "RAW"
    pub severity: String,
    pub desc: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalyticsProperties {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "kind")]
    pub kind: String,
    #[serde(rename = "cron")]
    pub cron: String,
    #[serde(rename = "channel-size")]
    pub channel_size: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LlmProperties {
    #[serde(rename = "api-url")]
    pub api_url: String,
    #[serde(rename = "api-key")]
    pub api_key: String,
    #[serde(rename = "model")]
    pub model: String,
    #[serde(rename = "system-prompt")]
    pub system_prompt: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForwardProperties {
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

impl AppConfigProperties {
    pub fn default() -> AppConfigProperties {
        AppConfigProperties {
            service_name: String::from("botwaf"),
            server: ServerProperties::default(),
            logging: LoggingProperties::default(),
            cache: CacheProperties::default(),
            botwaf: BotwafProperties::default(),
        }
    }
}

impl Default for ServerProperties {
    fn default() -> Self {
        ServerProperties {
            host: String::from("127.0.0.1"),
            port: 9999,
        }
    }
}

impl Default for LoggingProperties {
    fn default() -> Self {
        LoggingProperties {
            mode: LogMode::Json,
            level: "info".to_string(),
        }
    }
}

impl Default for CacheProperties {
    fn default() -> Self {
        CacheProperties {
            provider: CacheProvider::Memory,
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

impl Default for BotwafProperties {
    fn default() -> Self {
        BotwafProperties {
            blocked_status_code: None,
            blocked_header_name: String::from("X-BotWaf-Blocked"),
            allow_addition_modsec_info: true,
            static_rules: vec![],
            llm: LlmProperties::default(),
            analytics: Vec::new(),
            forward: ForwardProperties::default(),
        }
    }
}

impl Default for AnalyticsProperties {
    fn default() -> Self {
        AnalyticsProperties {
            name: String::from("default"),
            kind: String::from("SIMPLE_LLM"),
            cron: String::from("0/30 * * * * * *"), // Every half minute
            channel_size: 200,
        }
    }
}

impl Default for LlmProperties {
    fn default() -> Self {
        LlmProperties {
            api_url: String::from("https://dashscope.aliyuncs.com/compatible-mode/v1"),
            api_key: String::from("<your_api_key>"),
            model: String::from("qwen-plus"),
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
            http_proxy: None,
            connect_timeout: 5,
            read_timeout: 5,
            total_timeout: 10,
            verbose: false,
            upstream_destination_header_name: String::from("X-Upstream-Destination"),
        }
    }
}

pub fn init() -> Arc<AppConfigProperties> {
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
        println!("Loaded the config details: {}", serde_json::to_string(config.as_ref()).unwrap());
    }
    return config;
}

lazy_static! {
    pub static ref CFG: Arc<AppConfigProperties> = init();
}
