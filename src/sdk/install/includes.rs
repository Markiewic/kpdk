use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use tar::Archive;

use crate::error::{Error, Result};

use super::{download_checked, recreate_directory};

const PDK_INCLUDES_COMMIT: &str = "874918fea57549de86fc930bd787e97d705126c7";
const PDK_INCLUDES_URL: &str =
    "https://codeload.github.com/free-pdk/pdk-includes/tar.gz/874918fea57549de86fc930bd787e97d705126c7";
const PDK_INCLUDES_SHA256: &str =
    "3efebfa540f6d1d8115ea87f992449931ba82332f6e61f9b350ee38324f78c37";
const EASY_PDK_INCLUDES_COMMIT: &str = "3ccaf890816806003f0a534b245a240aae58772b";
const EASY_PDK_INCLUDES_URL: &str =
    "https://codeload.github.com/free-pdk/easy-pdk-includes/tar.gz/3ccaf890816806003f0a534b245a240aae58772b";
const EASY_PDK_INCLUDES_SHA256: &str =
    "16a7f3c728b8d47ad22657c547e8fc7469f5558bfbdb5907eb4e274bd12e5c80";

struct IncludePackage {
    name: &'static str,
    directory: &'static str,
    commit: &'static str,
    url: &'static str,
    sha256: &'static str,
}

const PACKAGES: [IncludePackage; 2] = [
    IncludePackage {
        name: "pdk-includes",
        directory: "pdk",
        commit: PDK_INCLUDES_COMMIT,
        url: PDK_INCLUDES_URL,
        sha256: PDK_INCLUDES_SHA256,
    },
    IncludePackage {
        name: "easy-pdk-includes",
        directory: "easy-pdk",
        commit: EASY_PDK_INCLUDES_COMMIT,
        url: EASY_PDK_INCLUDES_URL,
        sha256: EASY_PDK_INCLUDES_SHA256,
    },
];

pub(super) fn install(workspace: &Path, staging: &Path) -> Result<()> {
    for package in &PACKAGES {
        install_package(package, workspace, staging)?;
    }
    verify(staging)
}

pub(super) fn verify(root: &Path) -> Result<()> {
    let required = [
        root.join("pdk/device.h"),
        root.join("pdk/device/pfs154.h"),
        root.join("easy-pdk/calibrate.h"),
        root.join("easy-pdk/serial_num.h"),
    ];
    if let Some(path) = required.iter().find(|path| !path.is_file()) {
        Err(Error::Message(format!(
            "free-pdk include installation is missing `{}`",
            path.display()
        )))
    } else {
        Ok(())
    }
}

pub(super) fn append_manifest(contents: &mut String) {
    contents.push_str(&format!(
        "\n[pdk_includes]\ncommit = {PDK_INCLUDES_COMMIT:?}\nsource = {PDK_INCLUDES_URL:?}\nsha256 = {PDK_INCLUDES_SHA256:?}\n\n[easy_pdk_includes]\ncommit = {EASY_PDK_INCLUDES_COMMIT:?}\nsource = {EASY_PDK_INCLUDES_URL:?}\nsha256 = {EASY_PDK_INCLUDES_SHA256:?}\n"
    ));
}

fn install_package(package: &IncludePackage, workspace: &Path, staging: &Path) -> Result<()> {
    let archive_path = workspace.join(format!("{}.tar.gz", package.name));
    let unpack_root = workspace.join(format!("{}-unpacked", package.name));

    println!("Downloading {} at {}...", package.name, package.commit);
    download_checked(package.url, &archive_path, package.sha256)?;
    recreate_directory(&unpack_root)?;

    let file = File::open(&archive_path).map_err(|source| Error::Read {
        path: archive_path.clone(),
        source,
    })?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive.set_preserve_ownerships(false);
    archive.unpack(&unpack_root).map_err(|source| Error::Read {
        path: archive_path,
        source,
    })?;

    let source = single_directory(&unpack_root)?;
    let destination = staging.join(package.directory);
    fs::rename(&source, &destination).map_err(|source| Error::Write {
        path: destination,
        source,
    })
}

fn single_directory(root: &Path) -> Result<PathBuf> {
    let entries = fs::read_dir(root)
        .map_err(|source| Error::Read {
            path: root.to_owned(),
            source,
        })?
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|source| Error::Read {
            path: root.to_owned(),
            source,
        })?;
    let directories: Vec<PathBuf> = entries
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();

    if directories.len() == 1 {
        Ok(directories[0].clone())
    } else {
        Err(Error::Message(format!(
            "archive extracted into {} top-level directories instead of one",
            directories.len()
        )))
    }
}
