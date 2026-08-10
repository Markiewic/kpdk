mod examples;
mod includes;
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
    let include_root = sdk_root.join("include");
    let examples_root = sdk_root.join("examples");
    let sdcc_exe = executable(&sdcc_root.join("bin"), "sdcc");

    remove_legacy_programmer(&sdk_root)?;

    if sdcc_exe.is_file() && includes::verify(&include_root).is_ok() && !force {
        if examples::verify(&examples_root).is_err() {
            install_examples_only(&sdk_root, &examples_root)?;
        }
        write_manifest(&sdk_root, installer.as_ref())?;
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

    let process_id = std::process::id();
    let workspace = std::env::temp_dir().join(format!("kpdk-sdk-install-{process_id}"));
    recreate_directory(&workspace)?;
    let sdcc_staging = sdk_root.join(format!(".sdcc-staging-{process_id}"));
    let include_staging = sdk_root.join(format!(".include-staging-{process_id}"));
    let examples_staging = sdk_root.join(format!(".examples-staging-{process_id}"));
    recreate_directory(&sdcc_staging)?;
    recreate_directory(&include_staging)?;
    recreate_directory(&examples_staging)?;

    let install_result = installer
        .install(&workspace, &sdcc_staging)
        .and_then(|_| {
            verify_sdcc(
                &executable(&sdcc_staging.join("bin"), "sdcc"),
                distribution.version,
            )
        })
        .and_then(|_| includes::install(&workspace, &include_staging))
        .and_then(|_| examples::install(&workspace, &examples_staging));
    let _ = fs::remove_dir_all(&workspace);
    if install_result.is_err() {
        let _ = fs::remove_dir_all(&sdcc_staging);
        let _ = fs::remove_dir_all(&include_staging);
        let _ = fs::remove_dir_all(&examples_staging);
    }
    install_result?;

    replace_directory(&sdcc_staging, &sdcc_root)?;
    replace_directory(&include_staging, &include_root)?;
    replace_directory(&examples_staging, &examples_root)?;

    verify_sdcc(&sdcc_exe, distribution.version)?;
    includes::verify(&include_root)?;
    examples::verify(&examples_root)?;
    write_manifest(&sdk_root, installer.as_ref())?;
    println!("Installed kpdk SDK {SDK_VERSION} at {}", sdk_root.display());
    Ok(())
}

fn install_examples_only(sdk_root: &Path, examples_root: &Path) -> Result<()> {
    let process_id = std::process::id();
    let workspace = std::env::temp_dir().join(format!("kpdk-sdk-examples-{process_id}"));
    let staging = sdk_root.join(format!(".examples-staging-{process_id}"));
    recreate_directory(&workspace)?;
    recreate_directory(&staging)?;

    let result = examples::install(&workspace, &staging);
    let _ = fs::remove_dir_all(&workspace);
    if let Err(error) = result {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    replace_directory(&staging, examples_root)
}

fn remove_legacy_programmer(root: &Path) -> Result<()> {
    let bin = root.join("bin");
    for name in ["easypdkprog", "easypdkprog.exe", "easypdkprog-LICENSE"] {
        let path = bin.join(name);
        if path.is_file() {
            fs::remove_file(&path).map_err(|source| Error::Write {
                path: path.clone(),
                source,
            })?;
        }
    }

    if bin.is_dir() {
        let mut entries = fs::read_dir(&bin).map_err(|source| Error::Read {
            path: bin.clone(),
            source,
        })?;
        if entries.next().is_none() {
            fs::remove_dir(&bin).map_err(|source| Error::Write { path: bin, source })?;
        }
    }

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

fn replace_directory(staging: &Path, destination: &Path) -> Result<()> {
    if destination.exists() {
        fs::remove_dir_all(destination).map_err(|source| Error::Write {
            path: destination.to_owned(),
            source,
        })?;
    }
    fs::rename(staging, destination).map_err(|source| Error::Write {
        path: destination.to_owned(),
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
    includes::append_manifest(&mut contents);
    examples::append_manifest(&mut contents);
    fs::write(&path, contents).map_err(|source| Error::Write { path, source })
}

fn executable(directory: &Path, name: &str) -> PathBuf {
    directory.join(if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_only_legacy_programmer_files() {
        let temp = tempfile::tempdir().unwrap();
        let bin = temp.path().join("bin");
        fs::create_dir_all(&bin).unwrap();
        fs::write(bin.join("easypdkprog"), b"legacy").unwrap();
        fs::write(bin.join("easypdkprog.exe"), b"legacy").unwrap();
        fs::write(bin.join("easypdkprog-LICENSE"), b"legacy").unwrap();
        fs::write(bin.join("keep.txt"), b"user-owned").unwrap();

        remove_legacy_programmer(temp.path()).unwrap();

        assert!(!bin.join("easypdkprog").exists());
        assert!(!bin.join("easypdkprog.exe").exists());
        assert!(!bin.join("easypdkprog-LICENSE").exists());
        assert_eq!(fs::read(bin.join("keep.txt")).unwrap(), b"user-owned");
    }
}
