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

#[cfg(feature = "profiling-mem-prof")]
pub mod mem_prof_router {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use common_telemetry::{error, info};

    #[axum_macros::debug_handler]
    // () -> common_mem_prof::error::Result<impl IntoResponse>
    pub async fn mem_prof_handler() -> impl IntoResponse {
        info!("Dumping memory profile ... ");

        match common_mem_prof::dump_profile().await {
            Ok(result) => {
                info!("Finished dump memory file size: {}", result.len());
                Ok((StatusCode::OK, result))
            }
            Err(err) => {
                error!("Failed to dump memory profile: {:?}", err);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({"error": err.to_string()})),
                )
                    .into_response())
            }
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
            axum::Json(serde_json::json!({"error": "The 'profiling-mem-prof' feature is disabled"})),
        )
            .into_response()
    }
}

#[cfg(feature = "profiling-pprof")]
pub mod pprof_router {
    use axum::extract::Query;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use common_pprof::CPUProfiling;
    use common_telemetry::info;
    use serde::{Deserialize, Serialize};
    use std::num::NonZeroI32;
    use std::time::Duration;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum OutputFormat {
        /// google’s pprof format report in protobuf.
        Proto,
        Text,
        Flamegraph,
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(default)]
    pub struct PprofQuery {
        seconds: u64,
        frequency: NonZeroI32,
        format: OutputFormat,
    }

    impl Default for PprofQuery {
        fn default() -> PprofQuery {
            PprofQuery {
                seconds: 5,
                // Safety: 99 is non zero.
                frequency: NonZeroI32::new(99).unwrap(),
                format: OutputFormat::Proto,
            }
        }
    }

    #[axum_macros::debug_handler]
    pub async fn pprof_handler(Query(req): Query<PprofQuery>) -> impl IntoResponse {
        info!("Dumping pprof ... {:?}", req);

        let profiling = CPUProfiling::new(Duration::from_secs(req.seconds), req.frequency.into());
        let result = match req.format {
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
        (StatusCode::NOT_IMPLEMENTED, "The 'profiling-pprof' feature is disabled").into_response()
    }
}
