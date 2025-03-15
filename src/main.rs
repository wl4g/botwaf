use botwaf::{config::config, server::server, updater::updater};
use clap::{Arg, Command};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Command::new("Botwaf")
        .version(config::VERSION.as_str())
        .author("James Wong")
        .about(format!("Botwaf - A Mini Open Source AI WAF written in Rust.\n\n{}", config::VERSION.as_str()).to_owned())
        .arg_required_else_help(true) // When no args are provided, show help.
        //.help_template("{about}\n\n{usage-heading}\n\n{usage}\n\n{all-args}")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("PATH") // Tips for the user.
                .help("Set global configuration file")
                .global(true), // Global args are available to all subcommands.
        )
        .subcommand(
            Command::new("standalone").about("Run Botwaf All Components in One.").arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .action(clap::ArgAction::SetTrue)
                    .help("Verbose output."),
            ),
        )
        .subcommand(
            Command::new("serve").about("Run Botwaf ModSec Proxy Web Server.").arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .action(clap::ArgAction::SetTrue)
                    .help("Verbose output."),
            ),
        )
        .subcommand(
            Command::new("ingestor").about("Run Botwaf near Real-time Event data Ingestor.").arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .action(clap::ArgAction::SetTrue)
                    .help("Verbose output."),
            ),
        )
        .subcommand(
            Command::new("updater")
                .about("Run Botwaf based on AI LLM + Vector DB ModSec rules Updater.")
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .action(clap::ArgAction::SetTrue)
                        .help("Verbose output."),
                ),
        );

    let matches = app.get_matches();
    match matches.subcommand() {
        Some((name, sub_matches)) => match name {
            "serve" => {
                #[allow(unused)]
                let verbose = sub_matches.get_flag("verbose");
                server::start().await?;
            }
            "updater" => {
                #[allow(unused)]
                let verbose = sub_matches.get_flag("verbose");
                updater::start().await?;
            }
            _ => {
                eprintln!("Invalid subcommand: {}", name);
                std::process::exit(1);
            }
        },
        None => {
            tracing::info!("Use <command> --help for more information about a specific command.");
        }
    }

    Ok(())
}
