use std::ffi::OsStr;
use std::process::Command;

use crate::error::{Error, Result};

pub fn run<I, S>(program: &str, args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                Error::ToolMissing(program.to_owned())
            } else {
                Error::Message(format!("failed to start `{program}`: {source}"))
            }
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::Process {
            program: program.to_owned(),
            status: status.code().unwrap_or(-1),
        })
    }
}

pub fn version(program: &str, arg: &str) -> bool {
    Command::new(program)
        .arg(arg)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

