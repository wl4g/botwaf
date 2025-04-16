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

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use common_telemetry::tracing_subscriber::filter;
use common_telemetry::{info, RELOAD_HANDLE};
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum DynLogError {
    #[snafu(display("Invalid parameter: {}", reason))]
    InvalidParameter { reason: String },

    #[snafu(display("Internal error: {}", err_msg))]
    Internal { err_msg: String },
}

impl IntoResponse for DynLogError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            DynLogError::InvalidParameter { reason } => {
                (StatusCode::BAD_REQUEST, format!("Invalid parameter: {}", reason))
            }
            DynLogError::Internal { err_msg } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal error: {}", err_msg),
            ),
        };
        (status, message).into_response()
    }
}

#[axum_macros::debug_handler]
pub async fn dyn_log_handler(level: String) -> impl IntoResponse {
    let new_filter = level
        .parse::<filter::Targets>()
        .map_err(|e| DynLogError::InvalidParameter {
            reason: format!("Invalid filter \"{level}\": {e:?}"),
        })?;

    let mut old_filter = None;

    if let Some(reload_handle) = RELOAD_HANDLE.get() {
        reload_handle
            .modify(|filter| {
                old_filter = Some(filter.clone());
                *filter = new_filter.clone();
            })
            .map_err(|e| DynLogError::Internal {
                err_msg: format!("Fail to modify filter: {e:?}"),
            })?;
    } else {
        return Err(DynLogError::Internal {
            err_msg: "Reload handle not initialized".to_string(),
        });
    }

    let change_note = format!(
        "Log Level changed from {} to {}",
        old_filter.map(|f| f.to_string()).unwrap_or_default(),
        new_filter
    );

    info!("{}", change_note);

    Ok((StatusCode::OK, change_note))
}
