use std::ffi::{OsStr, OsString};
use std::process::Command;

use crate::error::{Error, Result};

pub struct CapturedOutput {
    pub stdout: String,
    pub stderr: String,
}

pub fn run<I, S>(program: &str, args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    run_with_env(program, args, &[])
}

pub fn run_with_env<I, S>(program: &str, args: I, envs: &[(OsString, OsString)]) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let status = Command::new(program)
        .args(args)
        .envs(envs.iter().map(|(key, value)| (key, value)))
        .status()
        .map_err(|source| process_start_error(program, source))?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::Process {
            program: program.to_owned(),
            status: status.code().unwrap_or(-1),
        })
    }
}

pub fn run_captured<I, S>(program: &str, args: I) -> Result<CapturedOutput>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    run_with_env_captured(program, args, &[])
}

pub fn run_with_env_captured<I, S>(
    program: &str,
    args: I,
    envs: &[(OsString, OsString)],
) -> Result<CapturedOutput>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(program)
        .args(args)
        .envs(envs.iter().map(|(key, value)| (key, value)))
        .output()
        .map_err(|source| process_start_error(program, source))?;

    let captured = CapturedOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    };

    if output.status.success() {
        Ok(captured)
    } else {
        let diagnostics = [captured.stderr.trim(), captured.stdout.trim()]
            .into_iter()
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        let suffix = if diagnostics.is_empty() {
            String::new()
        } else {
            format!(": {diagnostics}")
        };
        Err(Error::Message(format!(
            "`{program}` exited with status {}{suffix}",
            output.status.code().unwrap_or(-1)
        )))
    }
}

fn process_start_error(program: &str, source: std::io::Error) -> Error {
    if source.kind() == std::io::ErrorKind::NotFound {
        Error::ToolMissing(program.to_owned())
    } else {
        Error::Message(format!("failed to start `{program}`: {source}"))
    }
}

pub fn version(program: &str, arg: &str) -> bool {
    Command::new(program)
        .arg(arg)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
