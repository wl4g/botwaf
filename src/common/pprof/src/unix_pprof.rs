// Copyright 2023 Botwaf Team
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

use crate::error::{PprofSnafu, Result};
use pprof::protos::Message;
use snafu::ResultExt;
use std::time::Duration;

/// CPU profiler utility.
// Inspired by https://github.com/datafuselabs/databend/blob/67f445e83cd4eceda98f6c1c114858929d564029/src/common/base/src/base/profiling.rs
#[derive(Debug)]
pub struct CPUProfiling {
    /// Sample duration.
    duration: Duration,
    /// Sample frequency.
    frequency: i32,
}

impl CPUProfiling {
    /// Creates a new profiler.
    pub fn new(duration: Duration, frequency: i32) -> CPUProfiling {
        CPUProfiling { duration, frequency }
    }

    /// Profiles and returns a generated pprof report.
    pub async fn report(&self) -> Result<pprof::Report> {
        let guard = pprof::ProfilerGuardBuilder::default()
            .frequency(self.frequency)
            .blocklist(&["libc", "libgcc", "pthread", "vdso"])
            .build()
            .context(PprofSnafu)?;
        tokio::time::sleep(self.duration).await;
        guard.report().build().context(PprofSnafu)
    }

    /// Profiles and returns a generated text.
    pub async fn dump_text(&self) -> Result<String> {
        let report = self.report().await?;
        let text = format!("{report:?}");
        Ok(text)
    }

    /// Profiles and returns a generated flamegraph.
    pub async fn dump_flamegraph(&self) -> Result<Vec<u8>> {
        let mut body: Vec<u8> = Vec::new();

        let report = self.report().await?;
        report.flamegraph(&mut body).context(PprofSnafu)?;

        Ok(body)
    }

    /// Profiles and returns a generated proto.
    pub async fn dump_proto(&self) -> Result<Vec<u8>> {
        let report = self.report().await?;
        // Generate google’s pprof format report.
        let profile = report.pprof().context(PprofSnafu)?;
        let body = profile.encode_to_vec();

        Ok(body)
    }
}
