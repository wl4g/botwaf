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

use std::{
    fmt::{self, Display},
    io::LineWriter,
    str::FromStr,
};

use tracing::level_filters::LevelFilter;
use tracing_subscriber::{filter::Targets, EnvFilter, Layer};

use serde::{Deserialize, Serialize};

use crate::config::config;

pub type LogRouteHandle = tracing_subscriber::reload::Handle<LogRouteType, tracing_subscriber::Registry>;

pub type LogRouteType = tracing_subscriber::filter::Filtered<
    Option<Box<dyn tracing_subscriber::Layer<tracing_subscriber::Registry> + Send + Sync>>,
    Targets,
    tracing_subscriber::Registry,
>;

pub type SubscriberForSecondLayer = tracing_subscriber::layer::Layered<
    tracing_subscriber::reload::Layer<LogRouteType, tracing_subscriber::Registry>,
    tracing_subscriber::Registry,
>;

pub type LogStderrHandle = tracing_subscriber::reload::Handle<LogStderrType, SubscriberForSecondLayer>;

pub type LogStderrType = tracing_subscriber::filter::Filtered<
    Box<dyn tracing_subscriber::Layer<SubscriberForSecondLayer> + Send + Sync>,
    Targets,
    SubscriberForSecondLayer,
>;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogMode {
    #[default]
    Human,
    Json,
}

impl Display for LogMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogMode::Human => Display::fmt("HUMAN", f),
            LogMode::Json => Display::fmt("JSON", f),
        }
    }
}

impl FromStr for LogMode {
    type Err = LogModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "human" => Ok(LogMode::Human),
            "json" => Ok(LogMode::Json),
            _ => Err(LogModeError(s.to_owned())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Unsupported log mode level `{0}`. Supported values are `HUMAN` and `JSON`.")]
pub struct LogModeError(String);

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

#[derive(Debug)]
pub struct LogLevelError {
    pub given_log_level: String,
}

impl Display for LogLevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Log level '{}' is invalid. Accepted values are 'OFF', 'ERROR', 'WARN', 'INFO', 'DEBUG', and 'TRACE'.",
            self.given_log_level
        )
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Off => Display::fmt("OFF", f),
            LogLevel::Error => Display::fmt("ERROR", f),
            LogLevel::Warn => Display::fmt("WARN", f),
            LogLevel::Info => Display::fmt("INFO", f),
            LogLevel::Debug => Display::fmt("DEBUG", f),
            LogLevel::Trace => Display::fmt("TRACE", f),
        }
    }
}

impl std::error::Error for LogLevelError {}

impl FromStr for LogLevel {
    type Err = LogLevelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "off" => Ok(LogLevel::Off),
            "error" => Ok(LogLevel::Error),
            "warn" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            _ => Err(LogLevelError {
                given_log_level: s.to_owned(),
            }),
        }
    }
}

#[allow(unused)]
pub(super) fn default_log_route_layer() -> LogRouteType {
    None.with_filter(tracing_subscriber::filter::Targets::new().with_target("", LevelFilter::OFF))
}

#[allow(unused)]
pub(super) fn default_log_stderr_layer() -> LogStderrType {
    let layer = tracing_subscriber::fmt::layer()
        .with_writer(|| LineWriter::new(std::io::stderr()))
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE);

    let layer = match config::CFG.logging.mode {
        LogMode::Human => Box::new(layer) as Box<dyn tracing_subscriber::Layer<SubscriberForSecondLayer> + Send + Sync>,
        LogMode::Json => Box::new(layer.json()) as Box<dyn tracing_subscriber::Layer<SubscriberForSecondLayer> + Send + Sync>,
    };

    layer.with_filter(
        tracing_subscriber::filter::Targets::new().with_target("", LevelFilter::from_str(&config::CFG.logging.level.to_string()).unwrap()),
    )
}

#[allow(unused)]
pub(super) fn default_log_levels_layer() -> EnvFilter {
    EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "debug".into())
        // .add_directive("debug".parse().unwrap()) // default level.
        .add_directive("botwaf=debug".parse().unwrap())
        .add_directive("hyper=warn".parse().unwrap())
        .add_directive("tokio=trace".parse().unwrap()) // Notice: Must be at trace level to collect
}
