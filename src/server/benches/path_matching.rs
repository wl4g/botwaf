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

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use globset::{Glob, GlobSetBuilder};
use regex::Regex;
use std::time::Duration;

fn globset_match(_: &mut Criterion) {
    // Optional, set only when executing externally.
    let mut c = Criterion::default()
        .sample_size(1000)
        .measurement_time(Duration::from_secs(15)) // Test duration
        .warm_up_time(Duration::from_secs(5)); // Pre test duration

    let mut builder = GlobSetBuilder::new();
    builder.add(Glob::new("/public/**").unwrap());
    builder.add(Glob::new("/api/login").unwrap());
    let globset = builder.build().unwrap();

    c.bench_function("globset match", |b| {
        b.iter(|| {
            black_box(globset.is_match("/public/css/style.css"));
            black_box(globset.is_match("/api/login"));
            black_box(globset.is_match("/private/data"));
        })
    });
}

fn regex_match(_: &mut Criterion) {
    // Optional, set only when executing externally.
    let mut c = Criterion::default()
        .sample_size(1000)
        .measurement_time(Duration::from_secs(15)) // Test duration
        .warm_up_time(Duration::from_secs(5)); // Pre test duration

    let pattern = "^/public/.*$|^/api/login$";
    let re = Regex::new(pattern).unwrap();

    c.bench_function("regex match", |b| {
        b.iter(|| {
            black_box(re.is_match("/public/css/style.css"));
            black_box(re.is_match("/api/login"));
            black_box(re.is_match("/private/data"));
        })
    });
}

// Defintion the baseline test group.
criterion_group!(benches, globset_match, regex_match);

// Baseline test entrypoint.
criterion_main!(benches);
