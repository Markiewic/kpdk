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

const FREE_PDK_EXAMPLES_COMMIT: &str = "019c6204fef80bae6230ad299bcdf03f0f8a798f";
const FREE_PDK_EXAMPLES_SOURCE: &str = "https://github.com/free-pdk/free-pdk-examples";
const FREE_PDK_EXAMPLES_RAW_INCLUDE: &str =
    "https://raw.githubusercontent.com/free-pdk/free-pdk-examples/019c6204fef80bae6230ad299bcdf03f0f8a798f/include";

struct IncludePackage {
    name: &'static str,
    directory: &'static str,
    commit: &'static str,
    url: &'static str,
    sha256: &'static str,
}

struct HelperFile {
    name: &'static str,
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

const HELPER_FILES: [HelperFile; 5] = [
    HelperFile {
        name: "auto_sysclock.h",
        sha256: "5de524f6e400cea1444ef2469826886724fa83a5b443ebf376fd14e230909fb2",
    },
    HelperFile {
        name: "delay.h",
        sha256: "ff170c4a22fe0aed4a0c4950bf8b0898f881600e11e33b888f581d84f9bfd54b",
    },
    HelperFile {
        name: "millis.h",
        sha256: "2e22ffe853ede58f71d1025138e58030c8274527133126ad2b564650f9f73d64",
    },
    HelperFile {
        name: "serial.h",
        sha256: "ac13a5135401c3fea2c358ab1afb3f6595104b7c225ec11328f705fd803dfde7",
    },
    HelperFile {
        name: "startup.h",
        sha256: "011d440aebc6bb0fa01a63a3addade5b078f7908dea93827a3f09dff04e2381f",
    },
];

pub(super) fn install(workspace: &Path, staging: &Path) -> Result<()> {
    for package in &PACKAGES {
        install_package(package, workspace, staging)?;
    }
    install_example_helpers(staging)?;
    verify(staging)
}

pub(super) fn verify(root: &Path) -> Result<()> {
    let required = [
        root.join("pdk/device.h"),
        root.join("pdk/device/pfs154.h"),
        root.join("easy-pdk/calibrate.h"),
        root.join("easy-pdk/serial_num.h"),
        root.join("auto_sysclock.h"),
        root.join("delay.h"),
        root.join("millis.h"),
        root.join("serial.h"),
        root.join("startup.h"),
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
        "\n[pdk_includes]\ncommit = {PDK_INCLUDES_COMMIT:?}\nsource = {PDK_INCLUDES_URL:?}\nsha256 = {PDK_INCLUDES_SHA256:?}\n\n[easy_pdk_includes]\ncommit = {EASY_PDK_INCLUDES_COMMIT:?}\nsource = {EASY_PDK_INCLUDES_URL:?}\nsha256 = {EASY_PDK_INCLUDES_SHA256:?}\n\n[free_pdk_examples]\ncommit = {FREE_PDK_EXAMPLES_COMMIT:?}\nsource = {FREE_PDK_EXAMPLES_SOURCE:?}\n"
    ));
    for helper in &HELPER_FILES {
        contents.push_str(&format!(
            "\n[[free_pdk_examples.helpers]]\nname = {:?}\nsha256 = {:?}\n",
            helper.name, helper.sha256
        ));
    }
}

fn install_example_helpers(staging: &Path) -> Result<()> {
    println!("Downloading free-pdk-examples helpers at {FREE_PDK_EXAMPLES_COMMIT}...");
    for helper in &HELPER_FILES {
        let url = format!("{FREE_PDK_EXAMPLES_RAW_INCLUDE}/{}", helper.name);
        download_checked(&url, &staging.join(helper.name), helper.sha256)?;
    }
    Ok(())
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
