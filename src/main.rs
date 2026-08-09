mod commands;
mod config;
mod device;
mod error;
mod mcp;
mod process;
mod sdk;
mod toolchain;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::commands::{build, clean, doctor, flash, new, probe, vscode};
use crate::error::Result;
use crate::sdk::install;

#[derive(Parser)]
#[command(name = "kpdk", version, about = "A friendly frontend for free-pdk")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Install and manage the kpdk SDK.
    Sdk {
        #[command(subcommand)]
        command: SdkCommand,
    },
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
    /// Generate or refresh VS Code project integration.
    Vscode {
        #[arg(long, default_value = ".")]
        project: PathBuf,
    },
    /// Run the local MCP server over stdio.
    Mcp,
}

#[derive(Subcommand)]
enum SdkCommand {
    /// Install the pinned Windows SDCC toolchain.
    Install {
        #[arg(long)]
        force: bool,
    },
}

fn run() -> Result<()> {
    match Cli::parse().command {
        Command::Sdk { command: SdkCommand::Install { force } } => install::run(force),
        Command::New { name, device, clock, vdd } => new::run(&name, &device, clock, vdd),
        Command::Build { project, release } => {
            let artifacts = build::run(&project, release)?;
            build::print_artifacts(&artifacts);
            Ok(())
        }
        Command::Clean { project } => clean::run(&project),
        Command::Doctor => doctor::run(),
        Command::Probe => probe::run(),
        Command::Flash { project, port, yes } => flash::run(&project, port.as_deref(), yes),
        Command::Vscode { project } => vscode::run(&project),
        Command::Mcp => mcp::run(),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
