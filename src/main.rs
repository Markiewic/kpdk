mod commands;
mod config;
mod device;
mod error;
mod process;
mod toolchain;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::commands::{build, clean, doctor, flash, new, probe};
use crate::error::Result;

#[derive(Parser)]
#[command(name = "kpdk", version, about = "A friendly frontend for free-pdk")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new Padauk firmware project.
    New {
        name: String,
        #[arg(long)]
        device: String,
        #[arg(long, default_value_t = 8_000_000)]
        clock: u32,
        #[arg(long, default_value_t = 5_000)]
        vdd: u16,
    },
    /// Compile the current project.
    Build {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        release: bool,
    },
    /// Remove generated build artifacts.
    Clean {
        #[arg(long, default_value = ".")]
        project: PathBuf,
    },
    /// Check the host and free-pdk toolchain.
    Doctor,
    /// Ask Easy PDK Programmer to detect the connected IC.
    Probe,
    /// Build and program the current project.
    Flash {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        port: Option<String>,
        #[arg(long)]
        yes: bool,
    },
}

fn run() -> Result<()> {
    match Cli::parse().command {
        Command::New {
            name,
            device,
            clock,
            vdd,
        } => new::run(&name, &device, clock, vdd),
        Command::Build { project, release } => build::run(&project, release).map(|_| ()),
        Command::Clean { project } => clean::run(&project),
        Command::Doctor => doctor::run(),
        Command::Probe => probe::run(),
        Command::Flash { project, port, yes } => flash::run(&project, port.as_deref(), yes),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
