use std::fs;
use std::path::Path;
use std::process::Command;

use crate::error::{Error, Result};

use super::platform::{Distribution, Installer};
use super::{download_checked, executable, require_sdcc};

const SDCC_URL: &str =
    "https://sourceforge.net/projects/sdcc/files/sdcc-win64/4.6.0/sdcc-4.6.0-x64-setup.exe/download";
const SDCC_SHA256: &str = "0a165e155a052fcf7c29ea703ee77d5a8eb578eba58279e79e885618dc4b2e1a";

const SEVEN_ZIP_VERSION: &str = "26.02";
const SEVEN_ZIP_BOOTSTRAP_URL: &str = "https://www.7-zip.org/a/7zr.exe";
const SEVEN_ZIP_BOOTSTRAP_SHA256: &str =
    "56b8cc9f4971cef253644fafe54063ed7fdca551d4dee0f8c6baa81b855acd72";
const SEVEN_ZIP_URL: &str = "https://www.7-zip.org/a/7z2602-x64.exe";
const SEVEN_ZIP_SHA256: &str = "6745fa76dc2ea031596d8678f6f6b99c3c1b435b4164a63485adbbc7b8d82ef0";

pub(super) struct WindowsInstaller {
    distribution: Distribution,
}

impl WindowsInstaller {
    pub(super) fn new() -> Self {
        Self {
            distribution: Distribution {
                platform: "windows-x64",
                version: "4.6.0",
                channel: "stable",
                revision: None,
                url: SDCC_URL,
                sha256: SDCC_SHA256,
            },
        }
    }
}

impl Installer for WindowsInstaller {
    fn distribution(&self) -> &Distribution {
        &self.distribution
    }

    fn install(&self, workspace: &Path, staging: &Path) -> Result<()> {
        let seven_zip_bootstrap = workspace.join("7zr.exe");
        let seven_zip_installer = workspace.join("7z-x64.exe");
        let seven_zip_root = workspace.join("7zip");
        let sdcc_installer =
            workspace.join(format!("sdcc-{}-x64-setup.exe", self.distribution.version));

        println!("Downloading 7-Zip bootstrap...");
        download_checked(
            SEVEN_ZIP_BOOTSTRAP_URL,
            &seven_zip_bootstrap,
            SEVEN_ZIP_BOOTSTRAP_SHA256,
        )?;
        println!("Downloading 7-Zip {SEVEN_ZIP_VERSION}...");
        download_checked(SEVEN_ZIP_URL, &seven_zip_installer, SEVEN_ZIP_SHA256)?;

        // The official 7-Zip installer is a 7z self-extracting archive. 7zr can
        // unpack it without installing anything or changing the registry.
        extract(&seven_zip_bootstrap, &seven_zip_installer, &seven_zip_root)?;
        let seven_zip = executable(&seven_zip_root, "7z");
        let seven_zip_dll = seven_zip_root.join("7z.dll");
        if !seven_zip.is_file() || !seven_zip_dll.is_file() {
            return Err(Error::Message(
                "7-Zip bootstrap did not produce `7z.exe` and `7z.dll`".into(),
            ));
        }

        println!("Downloading SDCC {}...", self.distribution.version);
        download_checked(
            self.distribution.url,
            &sdcc_installer,
            self.distribution.sha256,
        )?;
        println!("Extracting relocatable SDCC into {}...", staging.display());
        extract(&seven_zip, &sdcc_installer, staging)?;
        require_sdcc(staging)
    }

    fn append_manifest(&self, contents: &mut String) {
        contents.push_str(&format!(
            "\n[seven_zip]\nversion = {SEVEN_ZIP_VERSION:?}\nbootstrap_source = {SEVEN_ZIP_BOOTSTRAP_URL:?}\nbootstrap_sha256 = {SEVEN_ZIP_BOOTSTRAP_SHA256:?}\nsource = {SEVEN_ZIP_URL:?}\nsha256 = {SEVEN_ZIP_SHA256:?}\n"
        ));
    }
}

fn extract(seven_zip: &Path, archive: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination).map_err(|source| Error::Write {
        path: destination.to_owned(),
        source,
    })?;
    let status = Command::new(seven_zip)
        .arg("x")
        .arg("-y")
        .arg(format!("-o{}", destination.display()))
        .arg(archive)
        .status()
        .map_err(|source| {
            Error::Message(format!(
                "failed to start archive extractor `{}`: {source}",
                seven_zip.display()
            ))
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::Process {
            program: seven_zip.display().to_string(),
            status: status.code().unwrap_or(-1),
        })
    }
}
