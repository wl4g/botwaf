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

#![allow(clippy::print_stdout)]

use clap::{Arg, Command};

fn main() {
    let app = Command::new("Botwaf for Benchmarks")
        .version("1.0.0")
        .author("James Wong")
        .about("Botwaf for Benchmarks")
        .arg_required_else_help(true) // When no args are provided, show help.
        .arg(
            Arg::new("thread_num")
                .short('t')
                .long("thread_num")
                .action(clap::ArgAction::SetTrue)
                .help("Benchmarks Thread Number."),
        );
    let matches = app.get_matches();

    // Getting the command-line arg for kind of long 'thread_num' with default 5
    let thread_num = matches.get_one::<usize>("thread_num").unwrap_or(&5).to_owned();

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(thread_num)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            println!("Begin benchmark...");
            // Some benchmark async code here...
            println!("End benchmark.");
        })
}
