use bzip2::read::BzDecoder;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use tar::Archive;

use crate::error::{Error, Result};

use super::platform::{Distribution, Installer};
use super::{download_checked, executable, require_sdcc};

const X64_SDCC_URL: &str =
    "https://sourceforge.net/projects/sdcc/files/sdcc-linux-amd64/4.6.0/sdcc-4.6.0-amd64-unknown-linux2.5.tar.bz2/download";
const X64_SDCC_SHA256: &str = "f6b929c62ed3082a26087885e0f1f9bf41878602ef1f57e40b11b4a01bf4f366";
const ARM64_SDCC_URL: &str = "https://sourceforge.net/projects/sdcc/files/snapshot_builds/aarch64-linux-gnu/sdcc-snapshot-aarch64-linux-gnu-20260728-16725.tar.bz2/download";
const ARM64_SDCC_SHA256: &str = "0cc1c7460007653efda17ccc479139fba0a2ae8f15f5d34553a4b377a336a85c";

pub(super) struct LinuxInstaller {
    distribution: Distribution,
}

impl LinuxInstaller {
    pub(super) fn x86_64() -> Self {
        Self {
            distribution: Distribution {
                platform: "linux-x64",
                version: "4.6.0",
                channel: "stable",
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
                version: "4.6.2",
                channel: "snapshot",
                revision: Some(16725),
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
        let archive_path = workspace.join("sdcc.tar.bz2");
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

        let file = File::open(&archive_path).map_err(|source| Error::Read {
            path: archive_path.clone(),
            source,
        })?;
        let decoder = BzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        archive.set_preserve_ownerships(false);
        archive.unpack(staging).map_err(|source| Error::Read {
            path: archive_path,
            source,
        })?;

        flatten_archive_root(staging)?;
        require_sdcc(staging)
    }
}

fn flatten_archive_root(staging: &Path) -> Result<()> {
    if executable(&staging.join("bin"), "sdcc").is_file() {
        return Ok(());
    }

    let entries = fs::read_dir(staging)
        .map_err(|source| Error::Read {
            path: staging.to_owned(),
            source,
        })?
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|source| Error::Read {
            path: staging.to_owned(),
            source,
        })?;
    let roots: Vec<PathBuf> = entries
        .iter()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && executable(&path.join("bin"), "sdcc").is_file())
        .collect();

    if roots.len() != 1 {
        return Err(Error::Message(
            "SDCC archive does not contain a single recognizable toolchain root".into(),
        ));
    }

    let root = &roots[0];
    let children = fs::read_dir(root)
        .map_err(|source| Error::Read {
            path: root.clone(),
            source,
        })?
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|source| Error::Read {
            path: root.clone(),
            source,
        })?;
    for child in children {
        let destination = staging.join(child.file_name());
        fs::rename(child.path(), &destination).map_err(|source| Error::Write {
            path: destination,
            source,
        })?;
    }
    fs::remove_dir(root).map_err(|source| Error::Write {
        path: root.clone(),
        source,
    })
}
