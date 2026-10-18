use clap::{Parser, Subcommand};
use std::process::ExitCode;

#[derive(Parser)]
#[command(version, about = "Controlled filesystem experiments")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Preflight,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Preflight => preflight(),
    }
}

fn preflight() -> ExitCode {
    if !cfg!(target_os = "linux") {
        eprintln!("preflight requires linux only.");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
