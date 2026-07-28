use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};
use crate::toolchain::default_sdk_root;

const SDK_VERSION: &str = "2026.1";
const SDCC_VERSION: &str = "4.6.0";
const SDCC_URL: &str = "https://sourceforge.net/projects/sdcc/files/sdcc-win64/4.6.0/sdcc-4.6.0-x64-setup.exe/download";
const SDCC_SHA256: &str = "0a165e155a052fcf7c29ea703ee77d5a8eb578eba58279e79e885618dc4b2e1a";

const SEVEN_ZIP_VERSION: &str = "26.02";
const SEVEN_ZIP_BOOTSTRAP_URL: &str = "https://www.7-zip.org/a/7zr.exe";
const SEVEN_ZIP_BOOTSTRAP_SHA256: &str =
    "56b8cc9f4971cef253644fafe54063ed7fdca551d4dee0f8c6baa81b855acd72";
const SEVEN_ZIP_URL: &str = "https://www.7-zip.org/a/7z2602-x64.exe";
const SEVEN_ZIP_SHA256: &str =
    "6745fa76dc2ea031596d8678f6f6b99c3c1b435b4164a63485adbbc7b8d82ef0";

pub fn run(force: bool) -> Result<()> {
    if !cfg!(target_os = "windows") {
        return Err(Error::Message(
            "`kpdk sdk install` currently supports Windows x64 only".into(),
        ));
    }
    if std::env::consts::ARCH != "x86_64" {
        return Err(Error::Message(
            "`kpdk sdk install` currently requires Windows x64".into(),
        ));
    }

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

    let result = install_sdcc(&workspace, &staging);
    let _ = fs::remove_dir_all(&workspace);
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result?;

    let staged_sdcc = executable(&staging.join("bin"), "sdcc");
    verify_sdcc(&staged_sdcc)?;

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

    verify_sdcc(&sdcc_exe)?;
    write_manifest(&sdk_root)?;
    println!("Installed kpdk SDK {SDK_VERSION} at {}", sdk_root.display());
    println!("Note: free-pdk includes and easypdkprog are not installed by this preview yet.");
    Ok(())
}

fn install_sdcc(workspace: &Path, staging: &Path) -> Result<()> {
    let seven_zip_bootstrap = workspace.join("7zr.exe");
    let seven_zip_installer = workspace.join("7z-x64.exe");
    let seven_zip_root = workspace.join("7zip");
    let sdcc_installer = workspace.join(format!("sdcc-{SDCC_VERSION}-x64-setup.exe"));

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

    println!("Downloading SDCC {SDCC_VERSION}...");
    download_checked(SDCC_URL, &sdcc_installer, SDCC_SHA256)?;
    println!("Extracting relocatable SDCC into {}...", staging.display());
    extract(&seven_zip, &sdcc_installer, staging)?;

    let sdcc = executable(&staging.join("bin"), "sdcc");
    if !sdcc.is_file() {
        return Err(Error::Message(format!(
            "SDCC archive was extracted but `{}` was not created",
            sdcc.display()
        )));
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

fn verify_sdcc(sdcc: &Path) -> Result<()> {
    let output = Command::new(sdcc)
        .arg("-v")
        .output()
        .map_err(|source| Error::Message(format!("failed to run extracted SDCC: {source}")))?;
    let version = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if output.status.success() && version.contains(SDCC_VERSION) {
        println!("{}", version.trim());
        Ok(())
    } else {
        Err(Error::Message(format!(
            "extracted SDCC did not report expected version {SDCC_VERSION}: {}",
            version.trim()
        )))
    }
}

fn write_manifest(root: &Path) -> Result<()> {
    let path = root.join("manifest.toml");
    let contents = format!(
        "sdk_version = {SDK_VERSION:?}\nplatform = \"windows-x64\"\n\n[sdcc]\nversion = {SDCC_VERSION:?}\nsource = {SDCC_URL:?}\nsha256 = {SDCC_SHA256:?}\n\n[seven_zip]\nversion = {SEVEN_ZIP_VERSION:?}\nbootstrap_source = {SEVEN_ZIP_BOOTSTRAP_URL:?}\nbootstrap_sha256 = {SEVEN_ZIP_BOOTSTRAP_SHA256:?}\nsource = {SEVEN_ZIP_URL:?}\nsha256 = {SEVEN_ZIP_SHA256:?}\n"
    );
    fs::write(&path, contents).map_err(|source| Error::Write { path, source })
}

fn executable(directory: &Path, name: &str) -> PathBuf {
    directory.join(if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    })
}
