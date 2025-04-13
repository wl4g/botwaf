// Copyright 2023 Greptime Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use axum::{routing, Router};

#[cfg(feature = "profiling-mem-prof")]
// #[allow(dead_code)]
pub mod mem_prof_router {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use common_telemetry::info;

    #[axum_macros::debug_handler]
    // () -> common_mem_prof::error::Result<impl IntoResponse>
    pub async fn mem_prof_handler() -> impl IntoResponse {
        info!("Dumping memory profile request... ");

        // use common_mem_prof::error::DumpProfileDataSnafu;
        // use snafu::ResultExt;
        // Ok((
        //     StatusCode::OK,
        //     common_mem_prof::dump_profile().await.context(DumpProfileDataSnafu)?,
        // ))

        match common_mem_prof::dump_profile().await {
            Ok(result) => {
                info!("Finished dump memory file size: {}", result.len());
                Ok((StatusCode::OK, result))
            }
            Err(err) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({"error": err.to_string()})),
            )
                .into_response()),
        }
    }
}

#[cfg(not(feature = "profiling-mem-prof"))]
pub mod mem_prof_router {
    use axum::{http::StatusCode, response::IntoResponse};
    #[axum_macros::debug_handler]
    pub async fn mem_prof_handler() -> impl IntoResponse {
        (
            StatusCode::NOT_IMPLEMENTED,
            axum::Json(serde_json::json!({"error": "The 'mem-prof' feature is disabled"})),
        )
            .into_response()
    }
}

#[cfg(feature = "profiling-pprof")]
pub mod pprof_router {
    use axum::extract::Query;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    // use common_pprof::error::{DumpPprofSnafu, Result};
    use common_pprof::CPUProfiling;
    use common_telemetry::info;
    use serde::{Deserialize, Serialize};
    // use snafu::ResultExt;
    use std::num::NonZeroI32;
    use std::time::Duration;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum OutputFormat {
        /// google’s pprof format report in protobuf.
        Proto,
        /// Simple text format.
        Text,
        /// svg flamegraph.
        Flamegraph,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(default)]
    pub struct PprofQuery {
        seconds: u64,
        frequency: NonZeroI32,
        output: OutputFormat,
    }

    impl Default for PprofQuery {
        fn default() -> PprofQuery {
            PprofQuery {
                seconds: 5,
                // Safety: 99 is non zero.
                frequency: NonZeroI32::new(99).unwrap(),
                output: OutputFormat::Proto,
            }
        }
    }

    #[axum_macros::debug_handler]
    pub async fn pprof_handler(Query(req): Query<PprofQuery>) -> impl IntoResponse {
        info!("Dumping pprof request... {:?}", req);

        let profiling = Profiling::new(Duration::from_secs(req.seconds), req.frequency.into());
        let result = match req.output {
            OutputFormat::Proto => profiling.dump_proto().await,
            OutputFormat::Text => profiling.dump_text().await.map(|r| r.into_bytes()),
            OutputFormat::Flamegraph => profiling.dump_flamegraph().await,
        };
        match result {
            Ok(body) => {
                info!("Finished dump pprof file size: {}", body.len());
                Ok((StatusCode::OK, body))
            }
            Err(err) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({"error": err.to_string()})),
            )
                .into_response()),
        }
    }
}

#[cfg(not(feature = "profiling-pprof"))]
pub mod pprof_router {
    use axum::{http::StatusCode, response::IntoResponse};
    #[axum_macros::debug_handler]
    pub async fn pprof_handler() -> impl IntoResponse {
        (StatusCode::NOT_IMPLEMENTED, "The 'pprof' feature is disabled").into_response()
    }
}

pub fn router() -> axum::Router {
    // Handlers for debug, we don't expect a timeout.
    Router::new().nest(
        "/debug",
        Router::new()
            // handler for changing log level dynamically.
            // .route("/log_level", routing::post(dyn_log::dyn_log_handler))
            .nest(
                "/prof",
                Router::new()
                    .route("/cpu", routing::post(pprof_router::pprof_handler))
                    .route("/mem", routing::post(mem_prof_router::mem_prof_handler)),
            ),
    )
}
