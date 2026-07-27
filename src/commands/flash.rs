use std::io::{self, Write};
use std::path::Path;

use crate::commands::build;
use crate::config::ProjectFile;
use crate::device;
use crate::error::{Error, Result};
use crate::process;
use crate::toolchain::Toolchain;

pub fn run(root: &Path, port: Option<&str>, yes: bool) -> Result<()> {
    let config = ProjectFile::load(root)?;
    if device::is_otp(&config.project.device) && !yes {
        print!(
            "{} is an OTP device and cannot be erased. Type the device name to continue: ",
            config.project.device
        );
        io::stdout()
            .flush()
            .map_err(|error| Error::Message(error.to_string()))?;
        let mut answer = String::new();
        io::stdin()
            .read_line(&mut answer)
            .map_err(|error| Error::Message(error.to_string()))?;
        if !answer.trim().eq_ignore_ascii_case(&config.project.device) {
            return Err(Error::Message("programming cancelled".into()));
        }
    }

    let artifacts = build::run(root, true)?;
    let toolchain = Toolchain::discover();
    let mut args = vec!["-n".to_owned(), config.project.device.to_ascii_uppercase()];
    let configured_port = port
        .map(str::to_owned)
        .or_else(|| config.programmer.port.filter(|value| value != "auto"));
    if let Some(port) = configured_port {
        args.extend(["-p".to_owned(), port]);
    }
    args.extend([
        "write".to_owned(),
        artifacts.ihx.to_string_lossy().into_owned(),
    ]);
    process::run(&toolchain.easypdkprog, args)
}
