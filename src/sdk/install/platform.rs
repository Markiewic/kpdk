use std::path::Path;

use crate::error::{Error, Result};

use super::linux::LinuxInstaller;
use super::windows::WindowsInstaller;

pub(super) struct Distribution {
    pub(super) platform: &'static str,
    pub(super) version: &'static str,
    pub(super) channel: &'static str,
    pub(super) revision: Option<u32>,
    pub(super) url: &'static str,
    pub(super) sha256: &'static str,
}

pub(super) trait Installer {
    fn distribution(&self) -> &Distribution;
    fn install(&self, workspace: &Path, staging: &Path) -> Result<()>;

    fn append_manifest(&self, _contents: &mut String) {}
}

pub(super) fn installer() -> Result<Box<dyn Installer>> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => Ok(Box::new(WindowsInstaller::new())),
        ("linux", "x86_64") => Ok(Box::new(LinuxInstaller::x86_64())),
        ("linux", "aarch64") => Ok(Box::new(LinuxInstaller::aarch64())),
        (os, arch) => Err(Error::Message(format!(
            "`kpdk sdk install` does not support {os}/{arch} yet; supported targets are Windows x64, Linux x64, and Linux ARM64"
        ))),
    }
}
