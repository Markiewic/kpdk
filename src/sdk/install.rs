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

    let sdk_root = default_sdk_root()
        .ok_or_else(|| Error::Message("cannot determine the local application data directory".into()))?;
    let sdcc_root = sdk_root.join("sdcc");
    let sdcc_exe = executable(&sdcc_root.join("bin"), "sdcc");

    if sdcc_exe.is_file() && !force {
        println!("SDK {SDK_VERSION} is already installed at {}", sdk_root.display());
        println!("Use --force to reinstall it.");
        return Ok(());
    }

    fs::create_dir_all(&sdk_root).map_err(|source| Error::Write {
        path: sdk_root.clone(),
        source,
    })?;

    let installer = std::env::temp_dir().join(format!("sdcc-{SDCC_VERSION}-x64-setup.exe"));
    println!("Downloading SDCC {SDCC_VERSION}...");
    let digest = download(SDCC_URL, &installer)?;
    println!("Downloaded SHA-256: {digest}");

    if sdcc_root.exists() {
        fs::remove_dir_all(&sdcc_root).map_err(|source| Error::Write {
            path: sdcc_root.clone(),
            source,
        })?;
    }

    println!("Installing SDCC into {}...", sdcc_root.display());
    install_silent(&installer, &sdcc_root)?;
    let _ = fs::remove_file(&installer);

    if !sdcc_exe.is_file() {
        return Err(Error::Message(format!(
            "SDCC installer completed but `{}` was not created",
            sdcc_exe.display()
        )));
    }

    verify_sdcc(&sdcc_exe)?;
    write_manifest(&sdk_root, &digest)?;
    println!("Installed kpdk SDK {SDK_VERSION} at {}", sdk_root.display());
    println!("Note: free-pdk includes and easypdkprog are not installed by this preview yet.");
    Ok(())
}

fn download(url: &str, destination: &Path) -> Result<String> {
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
        let count = response
            .read(&mut buffer)
            .map_err(|source| Error::Read {
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
    Ok(format!("{:x}", hasher.finalize()))
}

fn install_silent(installer: &Path, destination: &Path) -> Result<()> {
    let status = Command::new(installer)
        .arg("/S")
        // NSIS requires /D to be the final argument and does not use quotes.
        .arg(format!("/D={}", destination.display()))
        .status()
        .map_err(|source| Error::Message(format!("failed to start SDCC installer: {source}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::Process {
            program: installer.display().to_string(),
            status: status.code().unwrap_or(-1),
        })
    }
}

fn verify_sdcc(sdcc: &Path) -> Result<()> {
    let output = Command::new(sdcc)
        .arg("-v")
        .output()
        .map_err(|source| Error::Message(format!("failed to run installed SDCC: {source}")))?;
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
            "installed SDCC did not report expected version {SDCC_VERSION}: {}",
            version.trim()
        )))
    }
}

fn write_manifest(root: &Path, digest: &str) -> Result<()> {
    let path = root.join("manifest.toml");
    let contents = format!(
        "sdk_version = {SDK_VERSION:?}\nplatform = \"windows-x64\"\n\n[sdcc]\nversion = {SDCC_VERSION:?}\nsource = {SDCC_URL:?}\nsha256 = {digest:?}\n"
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
