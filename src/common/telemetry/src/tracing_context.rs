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

//! tracing stuffs, inspired by RisingWave
use std::collections::HashMap;

use opentelemetry::propagation::TextMapPropagator;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_opentelemetry::OpenTelemetrySpanExt;

// An wapper for `Futures` that provides tracing instrument adapters.
pub trait FutureExt: std::future::Future + Sized {
    fn trace(self, span: tracing::span::Span) -> tracing::instrument::Instrumented<Self>;
}

impl<T: std::future::Future> FutureExt for T {
    #[inline]
    fn trace(self, span: tracing::span::Span) -> tracing::instrument::Instrumented<Self> {
        tracing::instrument::Instrument::instrument(self, span)
    }
}

/// Context for tracing used for propagating tracing information in a distributed system.
///
/// Generally, the caller of a service should create a tracing context from the current tracing span
/// and pass it to the callee through the network. The callee will then attach its local tracing
/// span as a child of the tracing context, so that the external tracing service can associate them
/// in a single trace.
///
/// The tracing context must be serialized into the W3C trace context format and passed in rpc
/// message headers when communication of frontend, datanode and meta.
///
/// See [Trace Context](https://www.w3.org/TR/trace-context/) for more information.
#[derive(Debug, Clone)]
pub struct TracingContext(opentelemetry::Context);

pub type W3cTrace = HashMap<String, String>;

impl Default for TracingContext {
    fn default() -> Self {
        Self::new()
    }
}

type Propagator = TraceContextPropagator;

impl TracingContext {
    /// Create a new tracing context from a tracing span.
    pub fn from_span(span: &tracing::Span) -> Self {
        Self(span.context())
    }

    /// Create a new tracing context from the current tracing span considered by the subscriber.
    pub fn from_current_span() -> Self {
        Self::from_span(&tracing::Span::current())
    }

    /// Create a no-op tracing context.
    pub fn new() -> Self {
        Self(opentelemetry::Context::new())
    }

    /// Attach the given span as a child of the context. Returns the attached span.
    pub fn attach(&self, span: tracing::Span) -> tracing::Span {
        span.set_parent(self.0.clone());
        span
    }

    /// Convert the tracing context to the W3C trace context format.
    pub fn to_w3c(&self) -> W3cTrace {
        let mut fields = HashMap::new();
        Propagator::new().inject_context(&self.0, &mut fields);
        fields
    }

    /// Create a new tracing context from the W3C trace context format.
    pub fn from_w3c(fields: &W3cTrace) -> Self {
        let context = Propagator::new().extract(fields);
        Self(context)
    }

    /// Convert the tracing context to a JSON string in W3C trace context format.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.to_w3c()).unwrap()
    }

    /// Create a new tracing context from a JSON string in W3C trace context format.
    ///
    /// Illegal json string will produce an empty tracing context and no error will be reported.
    pub fn from_json(json: &str) -> Self {
        let fields: W3cTrace = serde_json::from_str(json).unwrap_or_default();
        Self::from_w3c(&fields)
    }
}
