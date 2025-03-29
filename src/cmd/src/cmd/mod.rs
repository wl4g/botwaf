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

pub mod management;
pub mod server;
pub mod standalone;
pub mod updater;
pub mod verifier;

use botwaf_server::config::config;
use clap::{Arg, ArgMatches, Command};
use server::WebServer;
use standalone::StandaloneServer;
use std::{collections::BTreeMap, sync::OnceLock};
use updater::UpdaterServer;
use verifier::VerifierServer;

/// TODO: Used jemalloc or tcmalloc as the default allocator for APM observe monitoring.
/// Check for the allocator used: 'objdump -t target/debug/botwaf | grep mi_os_alloc'
/// see:https://rustcc.cn/article?id=75f290cd-e8e9-4786-96dc-9a44e398c7f5
#[global_allocator]
// static GLOBAL: std::alloc::System = std::alloc::System;
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

type SubcommandBuildFn = fn() -> Command;
type SubcommandHandleFn = fn(&ArgMatches, bool) -> ();

static SUBCOMMAND_MAP: OnceLock<BTreeMap<&'static str, (SubcommandBuildFn, SubcommandHandleFn)>> = OnceLock::new();

pub fn register_subcommand_handles() -> &'static BTreeMap<&'static str, (SubcommandBuildFn, SubcommandHandleFn)> {
    SUBCOMMAND_MAP.get_or_init(|| {
        let mut map = BTreeMap::new();
        map.insert(
            WebServer::COMMAND_NAME,
            (
                // Type inference error, forced conversion need.
                WebServer::build as SubcommandBuildFn,
                WebServer::run as SubcommandHandleFn,
            ),
        );
        map.insert(
            StandaloneServer::COMMAND_NAME,
            (
                // Type inference error, forced conversion need.
                StandaloneServer::build as SubcommandBuildFn,
                StandaloneServer::run as SubcommandHandleFn,
            ),
        );
        map.insert(
            UpdaterServer::COMMAND_NAME,
            (
                // Type inference error, forced conversion need.
                UpdaterServer::build as SubcommandBuildFn,
                UpdaterServer::run as SubcommandHandleFn,
            ),
        );
        map.insert(
            VerifierServer::COMMAND_NAME,
            (
                // Type inference error, forced conversion need.
                VerifierServer::build as SubcommandBuildFn,
                VerifierServer::run as SubcommandHandleFn,
            ),
        );
        map
    })
}

pub fn execute_commands_app() -> () {
    let mut app = Command::new("Botwaf Rust Serve")
        .version(botwaf_server::config::config::VERSION.as_str())
        .author("James Wong")
        .about(
            format!(
                "Botwaf - A Mini Open Source AI Bot WAF written in Rust.\n\n{}",
                config::VERSION.as_str()
            )
            .to_owned(),
        )
        .arg_required_else_help(true) // When no args are provided, show help.
        //.help_template("{about}\n\n{usage-heading}\n\n{usage}\n\n{all-args}")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .value_name("PRINT") // Tips for the user.
                .help("Set up global details print flag")
                .global(true), // Global args are available to all subcommands.
        );

    let subcommand_map = register_subcommand_handles();
    // Add to all subcommands.
    for (name, (build_fn, _)) in subcommand_map.iter() {
        app = app.subcommand(build_fn().name(name));
    }

    let matches = app.get_matches();
    let verbose = matches.contains_id("verbose");

    // Handling to actual subcommand.
    match matches.subcommand() {
        Some((name, sub_matches)) => {
            if let Some(&(_, handler)) = subcommand_map.get(name) {
                tracing::info!("Executing subcommand: {}", name);
                handler(sub_matches, verbose);
            } else {
                // panic!("Unknown subcommand: {}. Use --help for a list of available commands.", name);
                eprintln!("Invalid commands and Use <command> --help for more information about a specific command.");
                std::process::exit(1);
            }
        }
        None => {
            tracing::info!("No subcommand was used. Available commands are:");
            for name in subcommand_map.keys() {
                tracing::info!("  {}", name);
            }
            tracing::info!("Use <command> --help for more information about a specific command.");
        }
    }
}
