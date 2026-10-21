use std::process::ExitCode;

use armada_fleet_print::cli::Cli;
use armada_fleet_print::error_chain::error_chain;
use clap::Parser;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> ExitCode {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    match Cli::parse().run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            tracing::error!("{}", error_chain(&err));
            ExitCode::FAILURE
        }
    }
}
