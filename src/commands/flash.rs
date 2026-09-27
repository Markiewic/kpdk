use std::path::Path;

use crate::commands::build;
use crate::config::ProjectFile;
use crate::error::Result;
use crate::process;
use crate::toolchain::Toolchain;

pub fn run(root: &Path, port: Option<&str>) -> Result<()> {
    let config = ProjectFile::load(root)?;
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
