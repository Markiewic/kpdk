use std::path::Path;

use crate::error::Result;

use super::platform::{Distribution, Installer};
use super::{download_checked, extract_tar_gz, require_sdcc};

const SDCC_URL: &str =
    "https://github.com/Markiewic/kpdk/releases/download/kpdk-toolchain-2026.1/sdcc-4.6.0-windows-x64.tar.gz";
const SDCC_SHA256: &str = "a18016353d0c7b6da3a2deb6b2d05588335d6757419b301ec9a2cc6f9f41a45d";

pub(super) struct WindowsInstaller {
    distribution: Distribution,
}

impl WindowsInstaller {
    pub(super) fn new() -> Self {
        Self {
            distribution: Distribution {
                platform: "windows-x64",
                version: "4.6.0",
                channel: "kpdk-toolchain",
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
        let archive_path = workspace.join("sdcc.tar.gz");
        println!("Downloading SDCC {}...", self.distribution.version);
        download_checked(
            self.distribution.url,
            &archive_path,
            self.distribution.sha256,
        )?;
        println!("Extracting relocatable SDCC into {}...", staging.display());
        extract_tar_gz(&archive_path, staging)?;
        require_sdcc(staging)
    }
}
