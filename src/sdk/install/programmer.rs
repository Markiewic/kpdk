use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};

use super::executable;

pub(super) const VERSION: &str = "1.3";
pub(super) const REVISION: &str = "c704defbf90a934d5d4969bded63e824048e0d24";
pub(super) const SOURCE: &str =
    "https://github.com/free-pdk/easy-pdk-programmer-software/tree/1.3";

pub(super) fn install(staging: &Path) -> Result<()> {
    fs::create_dir_all(staging).map_err(|source| Error::Write {
        path: staging.to_owned(),
        source,
    })?;

    let source = bundled_binary()?;
    let destination = executable(staging, "easypdkprog");
    fs::copy(&source, &destination).map_err(|error| Error::Write {
        path: destination.clone(),
        source: error,
    })?;
    verify_binary(&destination)
}

pub(super) fn verify(root: &Path) -> Result<()> {
    verify_binary(&executable(root, "easypdkprog"))
}

pub(super) fn append_manifest(contents: &mut String) {
    contents.push_str(&format!(
        "\n[easypdkprog]\nversion = {VERSION:?}\nrevision = {REVISION:?}\nsource = {SOURCE:?}\ndelivery = \"bundled-with-kpdk\"\n"
    ));
}

fn bundled_binary() -> Result<PathBuf> {
    if let Some(path) = env::var_os("KPDK_EASYPDKPROG").map(PathBuf::from) {
        if path.is_file() {
            return Ok(path);
        }
        return Err(Error::Message(format!(
            "KPDK_EASYPDKPROG points to missing file `{}`",
            path.display()
        )));
    }

    let current_exe = env::current_exe()
        .map_err(|source| Error::Message(format!("cannot locate the kpdk executable: {source}")))?;
    let directory = current_exe.parent().ok_or_else(|| {
        Error::Message(format!(
            "cannot determine the directory containing `{}`",
            current_exe.display()
        ))
    })?;
    let bundled = executable(directory, "easypdkprog");
    if bundled.is_file() {
        Ok(bundled)
    } else {
        Err(Error::Message(format!(
            "bundled easypdkprog was not found at `{}`; install kpdk from the complete release archive or set KPDK_EASYPDKPROG",
            bundled.display()
        )))
    }
}

fn verify_binary(program: &Path) -> Result<()> {
    if !program.is_file() {
        return Err(Error::Message(format!(
            "easypdkprog was not installed at `{}`",
            program.display()
        )));
    }

    let output = Command::new(program)
        .arg("--version")
        .output()
        .map_err(|source| {
            Error::Message(format!(
                "failed to run installed easypdkprog `{}`: {source}",
                program.display()
            ))
        })?;
    let version = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if output.status.success() && version.contains(VERSION) {
        println!("{}", version.trim());
        Ok(())
    } else {
        Err(Error::Message(format!(
            "installed easypdkprog did not report expected version {VERSION}: {}",
            version.trim()
        )))
    }
}
