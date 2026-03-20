use daily_briefing::config;
use daily_briefing::runner;

use clap::{Parser, Subcommand};
use std::process;
use tracing::error;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[clap(
    name = "daily-briefing",
    about = "Collect, process and summarize daily inputs"
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Execute the full pipeline: collect inputs, process, write outputs
    Run {
        /// Path to the configuration file
        #[clap(short, long, default_value = "config.toml")]
        config: String,
    },
    /// Parse and validate config, then exit without running the pipeline
    Validate {
        /// Path to the configuration file
        #[clap(short, long, default_value = "config.toml")]
        config: String,
    },
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { config } => match config::Config::new(&config) {
            Ok(cfg) => {
                println!("Config is valid.");
                println!("  Inputs:    {}", cfg.inputs.len());
                println!("  Processor: {}", cfg.processor.type_name());
                println!("  Outputs:   {}", cfg.outputs.len());
                process::exit(exitcode::OK);
            }
            Err(e) => {
                error!("Config error: {}", e);
                process::exit(exitcode::CONFIG);
            }
        },
        Commands::Run { config } => {
            let cfg = match config::Config::new(&config) {
                Ok(c) => c,
                Err(e) => {
                    error!("Config error: {}", e);
                    process::exit(exitcode::CONFIG);
                }
            };

            if let Err(e) = runner::run(cfg).await {
                error!("Pipeline error: {}", e);
                process::exit(exitcode::DATAERR);
            }

            process::exit(exitcode::OK);
        }
    }
}
