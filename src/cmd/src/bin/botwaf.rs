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

use botwaf_cmd::cmd;

// Check for the allocator used: 'objdump -t target/debug/botwaf | grep mi_os_alloc'
// see:https://rustcc.cn/article?id=75f290cd-e8e9-4786-96dc-9a44e398c7f5
// #[global_allocator]
// static GLOBAL: std::alloc::System = std::alloc::System;
// static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// Used jemalloc/tcmalloc as the default allocator for APM observe monitoring.
#[cfg(not(windows))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

pub fn main() {
    cmd::execute_commands_app();
}
