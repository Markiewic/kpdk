mod commands;
mod config;
mod device;
mod error;
mod mcp;
mod process;
mod sdk;
mod toolchain;

use clap::{Parser, Subcommand};
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use crate::commands::{build, clean, doctor, flash, new, probe, vscode};
use crate::error::{Error, Result};
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
        /// Install a missing SDK without prompting.
        #[arg(long)]
        yes: bool,
    },
    /// Remove generated build artifacts.
    Clean {
        #[arg(long, default_value = ".")]
        project: PathBuf,
    },
    /// Check the host and free-pdk toolchain.
    Doctor,
    /// Run the local MCP server over stdio.
    Mcp,
    /// Ask Easy PDK Programmer to detect the connected IC.
    Probe,
    /// Build and program the current project.
    Flash {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        port: Option<String>,
    },
    /// Generate or refresh VS Code project integration.
    Vscode {
        #[arg(long, default_value = ".")]
        project: PathBuf,
    },
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
        Command::Sdk {
            command: SdkCommand::Install { force },
        } => install::run(force),
        Command::New {
            name,
            device,
            clock,
            vdd,
        } => new::run(&name, &device, clock, vdd),
        Command::Build {
            project,
            release,
            yes,
        } => {
            ensure_sdk(yes)?;
            build::run(&project, release).map(|_| ())
        }
        Command::Clean { project } => clean::run(&project),
        Command::Doctor => doctor::run(),
        Command::Mcp => mcp::run(),
        Command::Probe => probe::run(),
        Command::Flash { project, port } => flash::run(&project, port.as_deref()),
        Command::Vscode { project } => vscode::run(&project),
    }
}

fn ensure_sdk(yes: bool) -> Result<()> {
    if crate::toolchain::sdk_is_installed() {
        return Ok(());
    }
    if yes {
        return install::run(false);
    }
    if !io::stdin().is_terminal() {
        return Err(Error::Message(
            "the kpdk SDK is not installed; run `kpdk sdk install` or retry with `kpdk build --yes`".into(),
        ));
    }

    print!("The kpdk SDK is required to build this project. Install it now? [Y/n] ");
    io::stdout()
        .flush()
        .map_err(|error| Error::Message(error.to_string()))?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|error| Error::Message(error.to_string()))?;

    match answer.trim().to_ascii_lowercase().as_str() {
        "" | "y" | "yes" => install::run(false),
        _ => Err(Error::Message(
            "SDK installation skipped; run `kpdk sdk install` before building".into(),
        )),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
