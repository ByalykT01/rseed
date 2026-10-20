use clap::{Parser, Subcommand};
use std::env::{self};
use std::path::PathBuf;
use std::process::ExitCode;

const REQUIRED_TOOLS: &[&str] = &[
    "fallocate",
    "mkfs.ext4",
    "mount",
    "umount",
    "findmnt",
    "df",
    "gzip",
    "strace",
    "timeout",
];

#[derive(Parser)]
#[command(version, about = "Controlled filesystem experiments")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Preflight,
    Prepare {
        #[arg(long)]
        run_dir: PathBuf,
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
        size_mib: u64,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Preflight => preflight(),
        Commands::Prepare { run_dir, size_mib } => prepare(&run_dir, size_mib),
    }
}

fn prepare(run_dir: &std::path::Path, size_mib: u64) -> ExitCode {
    let Some(size_bytes) = size_mib.checked_mul(1_048_576) else {
        eprintln!("image size is too large!");
        return ExitCode::FAILURE;
    };

    match std::fs::create_dir(&run_dir) {
        Ok(()) => {
            println!("Created run directory: {}", run_dir.display());
            println!("Planned image size: {} bytes", size_bytes);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!(
                "Couldnt create a run directory {}: {error}",
                run_dir.display()
            );
            ExitCode::FAILURE
        }
    }
}

fn preflight() -> ExitCode {
    if !cfg!(target_os = "linux") {
        eprintln!("preflight requires linux only.");
        return ExitCode::FAILURE;
    }

    let mut missing_count = 0;
    for tool in REQUIRED_TOOLS {
        match find_tool(tool) {
            Some(path) => {
                println!("Found tool {}", path.display());
            }
            None => {
                eprintln!("{} was not found", tool);
                missing_count += 1;
            }
        }
    }

    if missing_count > 0 {
        eprintln!("{missing_count} required tool(s) missing");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn find_tool(name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;

    for directory in env::split_paths(&path) {
        let candidate = directory.join(name);

        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}
