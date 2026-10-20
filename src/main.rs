use clap::{Parser, Subcommand};
use std::env::{self};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

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

    let result = match cli.command {
        Commands::Preflight => preflight(),
        Commands::Prepare { run_dir, size_mib } => prepare(&run_dir, size_mib),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn allocate_image(size_bytes: u64, image_path: &Path) -> io::Result<()> {
    let status = Command::new("fallocate")
        .arg("-l")
        .arg(size_bytes.to_string())
        .arg("--")
        .arg(image_path)
        .status()?;

    if !status.success() {
        return Err(io::Error::other(format!(
            "fallocate failed for {}: {status}",
            image_path.display()
        )));
    }

    Ok(())
}

fn format_ext4(image_path: &Path) -> io::Result<()> {
    let status = Command::new("mkfs.ext4")
        .arg("-F")
        .arg("--")
        .arg(image_path)
        .status()?;

    if !status.success() {
        return Err(io::Error::other(format!(
            "mkfs.ext4 failed for {}: {status}",
            image_path.display()
        )));
    }

    Ok(())
}

fn prepare(run_dir: &Path, size_mib: u64) -> io::Result<()> {
    let size_bytes = size_mib
        .checked_mul(1_048_576)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "image size is too large!"))?;

    std::fs::create_dir(run_dir)?;
    println!("Created run directory: {}", run_dir.display());

    let image_path = run_dir.join("disk.img");

    allocate_image(size_bytes, image_path.as_path())?;
    println!(
        "Allocated disk image: {} ({size_bytes} bytes)",
        image_path.display()
    );
    format_ext4(&image_path)?;
    println!("Formatted disk image: {}", image_path.display());
    Ok(())
}

fn preflight() -> io::Result<()> {
    if !cfg!(target_os = "linux") {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "preflight requires linux only.",
        ));
    }

    let mut missing_count: u8 = 0;
    for tool in REQUIRED_TOOLS {
        if let Some(path) = find_tool(tool) {
            println!("Found tool {}", path.display());
        } else {
            eprintln!("{tool} was not found");
            missing_count = missing_count.saturating_add(1);
        }
    }

    if missing_count > 0 {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{missing_count} required tool(s) missing"),
        ));
    }

    Ok(())
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
