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

pub mod dyn_log;
pub mod profiling;

use axum::{routing, Router};
use profiling::{mem_prof_router, pprof_router};

pub fn router() -> axum::Router {
    // Handlers for debug, we don't expect a timeout.
    Router::new().nest(
        "/debug",
        Router::new()
            // handler for changing log level dynamically.
            .route("/log_level", routing::post(dyn_log::dyn_log_handler))
            .nest(
                "/prof",
                Router::new()
                    .route("/cpu", routing::post(pprof_router::pprof_handler))
                    .route("/mem", routing::post(mem_prof_router::mem_prof_handler)),
            ),
    )
}
