mod linux;
mod platform;
mod windows;

use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};
use crate::toolchain::default_sdk_root;

use platform::Installer;

const SDK_VERSION: &str = "2026.1";

pub fn run(force: bool) -> Result<()> {
    let installer = platform::installer()?;
    let distribution = installer.distribution();
    let sdk_root = default_sdk_root().ok_or_else(|| {
        Error::Message("cannot determine the local application data directory".into())
    })?;
    let sdcc_root = sdk_root.join("sdcc");
    let sdcc_exe = executable(&sdcc_root.join("bin"), "sdcc");

    if sdcc_exe.is_file() && !force {
        println!(
            "SDK {SDK_VERSION} is already installed at {}",
            sdk_root.display()
        );
        println!("Use --force to reinstall it.");
        return Ok(());
    }

    fs::create_dir_all(&sdk_root).map_err(|source| Error::Write {
        path: sdk_root.clone(),
        source,
    })?;

    let workspace = std::env::temp_dir().join(format!("kpdk-sdk-install-{}", std::process::id()));
    recreate_directory(&workspace)?;
    let staging = sdk_root.join(format!(".sdcc-staging-{}", std::process::id()));
    recreate_directory(&staging)?;

    let install_result = installer.install(&workspace, &staging).and_then(|_| {
        verify_sdcc(
            &executable(&staging.join("bin"), "sdcc"),
            distribution.version,
        )
    });
    let _ = fs::remove_dir_all(&workspace);
    if install_result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    install_result?;

    if sdcc_root.exists() {
        fs::remove_dir_all(&sdcc_root).map_err(|source| Error::Write {
            path: sdcc_root.clone(),
            source,
        })?;
    }
    fs::rename(&staging, &sdcc_root).map_err(|source| Error::Write {
        path: sdcc_root.clone(),
        source,
    })?;

    verify_sdcc(&sdcc_exe, distribution.version)?;
    write_manifest(&sdk_root, installer.as_ref())?;
    println!("Installed kpdk SDK {SDK_VERSION} at {}", sdk_root.display());
    println!("Note: free-pdk includes and easypdkprog are not installed by this preview yet.");
    Ok(())
}

fn recreate_directory(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|source| Error::Write {
            path: path.to_owned(),
            source,
        })?;
    }
    fs::create_dir_all(path).map_err(|source| Error::Write {
        path: path.to_owned(),
        source,
    })
}

fn download_checked(url: &str, destination: &Path, expected_sha256: &str) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!("kpdk/", env!("CARGO_PKG_VERSION")))
        .build()?;
    let mut response = client.get(url).send()?.error_for_status()?;
    let mut file = File::create(destination).map_err(|source| Error::Write {
        path: destination.to_owned(),
        source,
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let count = response.read(&mut buffer).map_err(|source| Error::Read {
            path: destination.to_owned(),
            source,
        })?;
        if count == 0 {
            break;
        }
        file.write_all(&buffer[..count])
            .map_err(|source| Error::Write {
                path: destination.to_owned(),
                source,
            })?;
        hasher.update(&buffer[..count]);
    }
    drop(file);

    let actual_sha256 = format!("{:x}", hasher.finalize());
    if actual_sha256 != expected_sha256 {
        let _ = fs::remove_file(destination);
        return Err(Error::Message(format!(
            "SHA-256 mismatch for {url}: expected {expected_sha256}, got {actual_sha256}"
        )));
    }
    Ok(())
}

fn require_sdcc(root: &Path) -> Result<()> {
    let sdcc = executable(&root.join("bin"), "sdcc");
    if sdcc.is_file() {
        Ok(())
    } else {
        Err(Error::Message(format!(
            "SDCC archive was extracted but `{}` was not created",
            sdcc.display()
        )))
    }
}

fn verify_sdcc(sdcc: &Path, expected_version: &str) -> Result<()> {
    let output = Command::new(sdcc)
        .arg("-v")
        .output()
        .map_err(|source| Error::Message(format!("failed to run extracted SDCC: {source}")))?;
    let version = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if output.status.success() && version.contains(expected_version) {
        println!("{}", version.trim());
        Ok(())
    } else {
        Err(Error::Message(format!(
            "extracted SDCC did not report expected version {expected_version}: {}",
            version.trim()
        )))
    }
}

fn write_manifest(root: &Path, installer: &dyn Installer) -> Result<()> {
    let distribution = installer.distribution();
    let path = root.join("manifest.toml");
    let mut contents = format!(
        "sdk_version = {SDK_VERSION:?}\nplatform = {:?}\n\n[sdcc]\nversion = {:?}\nchannel = {:?}\nsource = {:?}\nsha256 = {:?}\n",
        distribution.platform,
        distribution.version,
        distribution.channel,
        distribution.url,
        distribution.sha256
    );
    if let Some(revision) = distribution.revision {
        contents.push_str(&format!("revision = {revision}\n"));
    }
    installer.append_manifest(&mut contents);
    fs::write(&path, contents).map_err(|source| Error::Write { path, source })
}

fn executable(directory: &Path, name: &str) -> PathBuf {
    directory.join(if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    })
}
