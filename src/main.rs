use botwaf::{config::config, server::server, updater::updater, verifier::verifier};
use clap::{Arg, Command};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Command::new("Botwaf")
        .version(config::VERSION.as_str())
        .author("James Wong")
        .about(format!("Botwaf - A Mini Open Source AI Bot WAF written in Rust.\n\n{}", config::VERSION.as_str()).to_owned())
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
        .subcommand(Command::new("standalone").about("Run Botwaf All Components in One."))
        .subcommand(Command::new("serve").about("Run Botwaf ModSec proxy Web Server."))
        .subcommand(Command::new("verifier").about("Run Botwaf AI generated ModSec rules Verifier."))
        .subcommand(Command::new("updater").about("Run Botwaf based on AI LLM + Vector DB ModSec rules Updater."));

    let matches = app.get_matches();
    match matches.subcommand() {
        Some((name, _)) => match name {
            "standalone" => {
                // TODO When server started blocking exeuction problem.
                server::start().await?;
                updater::start().await?;
                verifier::start().await?;
            }
            "serve" => {
                server::start().await?;
            }
            "verifier" => {
                verifier::start().await?;
            }
            "updater" => {
                updater::start().await?;
            }
            _ => {
                eprintln!("Invalid commands and Use <command> --help for more information about a specific command.");
                std::process::exit(1);
            }
        },
        None => {
            eprintln!("Use <command> --help for more information about a specific command.");
        }
    }

    Ok(())
}
