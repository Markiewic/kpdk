use std::path::Path;

use crate::error::Result;

use super::platform::{Distribution, Installer};
use super::{download_checked, extract_tar_gz, require_sdcc};

const X64_SDCC_URL: &str = "https://github.com/Markiewic/kpdk/releases/download/kpdk-toolchain-2026.1/sdcc-4.6.0-linux-x64.tar.gz";
const X64_SDCC_SHA256: &str = "b007aa20d409411e5c4df75cbbc2350e14f59889e78c2404c7089014b771649b";
const ARM64_SDCC_URL: &str = "https://github.com/Markiewic/kpdk/releases/download/kpdk-toolchain-2026.1/sdcc-4.6.3-r16925-linux-arm64.tar.gz";
const ARM64_SDCC_SHA256: &str = "142431edddca1ab3d81a8c03178fe19a5e87e6ca4e8d70fc7b98548b0700a9c1";

pub(super) struct LinuxInstaller {
    distribution: Distribution,
}

impl LinuxInstaller {
    pub(super) fn x86_64() -> Self {
        Self {
            distribution: Distribution {
                platform: "linux-x64",
                version: "4.6.0",
                channel: "kpdk-toolchain",
                revision: None,
                url: X64_SDCC_URL,
                sha256: X64_SDCC_SHA256,
            },
        }
    }

    pub(super) fn aarch64() -> Self {
        Self {
            distribution: Distribution {
                platform: "linux-arm64",
                version: "4.6.3",
                channel: "kpdk-toolchain",
                revision: Some(16925),
                url: ARM64_SDCC_URL,
                sha256: ARM64_SDCC_SHA256,
            },
        }
    }
}

impl Installer for LinuxInstaller {
    fn distribution(&self) -> &Distribution {
        &self.distribution
    }

    fn install(&self, workspace: &Path, staging: &Path) -> Result<()> {
        let archive_path = workspace.join("sdcc.tar.gz");
        println!(
            "Downloading SDCC {} ({})...",
            self.distribution.version, self.distribution.channel
        );
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
