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

use common_error::ext::ErrorExt;
use common_error::status_code::StatusCode;
use common_macro::stack_trace_debug;
use common_telemetry::error;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use prometheus::*;
use snafu::{Location, ResultExt, Snafu};
use std::any::Any;
use tikv_jemalloc_ctl::stats::{allocated_mib, resident_mib};
use tikv_jemalloc_ctl::{epoch, epoch_mib, stats};

lazy_static! {
    pub static ref SYS_JEMALLOC_RESIDEN: IntGauge = register_int_gauge!(
        "sys_jemalloc_resident",
        "Total number of bytes allocated by the application."
    )
    .unwrap();
    pub static ref SYS_JEMALLOC_ALLOCATED: IntGauge = register_int_gauge!(
        "sys_jemalloc_allocated",
        "Total number of bytes in physically resident data pages mapped by the allocator."
    )
    .unwrap();
}

pub(crate) static JEMALLOC_COLLECTOR: Lazy<Option<JemallocCollector>> = Lazy::new(|| {
    let collector = JemallocCollector::try_new()
        .map_err(|e| {
            error!(e; "Failed to retrieve jemalloc metrics");
            e
        })
        .ok();
    collector.inspect(|c| {
        if let Err(e) = c.update() {
            error!(e; "Failed to update jemalloc metrics");
        };
    })
});

pub type Result<T> = std::result::Result<T, JemallocError>;

pub(crate) struct JemallocCollector {
    epoch: epoch_mib,
    allocated: allocated_mib,
    resident: resident_mib,
}

impl JemallocCollector {
    pub(crate) fn try_new() -> Result<Self> {
        let e = epoch::mib().context(UpdateJemallocMetricsSnafu)?;
        let allocated = stats::allocated::mib().context(UpdateJemallocMetricsSnafu)?;
        let resident = stats::resident::mib().context(UpdateJemallocMetricsSnafu)?;
        Ok(Self {
            epoch: e,
            allocated,
            resident,
        })
    }

    pub(crate) fn update(&self) -> Result<()> {
        let _ = self.epoch.advance().context(UpdateJemallocMetricsSnafu)?;
        let allocated = self.allocated.read().context(UpdateJemallocMetricsSnafu)?;
        let resident = self.resident.read().context(UpdateJemallocMetricsSnafu)?;
        SYS_JEMALLOC_RESIDEN.set(allocated as i64);
        SYS_JEMALLOC_ALLOCATED.set(resident as i64);
        Ok(())
    }
}

#[derive(Snafu)]
#[snafu(visibility(pub))]
#[stack_trace_debug]
pub enum JemallocError {
    #[cfg(not(windows))]
    #[snafu(display("Failed to update jemalloc metrics"))]
    UpdateJemallocMetrics {
        #[snafu(source)]
        error: tikv_jemalloc_ctl::Error,
        #[snafu(implicit)]
        location: Location,
    },
}

impl ErrorExt for JemallocError {
    // Use renamed trait
    fn status_code(&self) -> StatusCode {
        match self {
            #[cfg(not(windows))]
            JemallocError::UpdateJemallocMetrics { .. } => StatusCode::Internal,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
