use bzip2::read::BzDecoder;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Archive;

use crate::error::{Error, Result};
use crate::toolchain::default_sdk_root;

const SDK_VERSION: &str = "2026.1";

const WINDOWS_SDCC_URL: &str =
    "https://sourceforge.net/projects/sdcc/files/sdcc-win64/4.6.0/sdcc-4.6.0-x64-setup.exe/download";
const WINDOWS_SDCC_SHA256: &str =
    "0a165e155a052fcf7c29ea703ee77d5a8eb578eba58279e79e885618dc4b2e1a";
const LINUX_X64_SDCC_URL: &str =
    "https://sourceforge.net/projects/sdcc/files/sdcc-linux-amd64/4.6.0/sdcc-4.6.0-amd64-unknown-linux2.5.tar.bz2/download";
const LINUX_X64_SDCC_SHA256: &str =
    "f6b929c62ed3082a26087885e0f1f9bf41878602ef1f57e40b11b4a01bf4f366";
const LINUX_ARM64_SDCC_URL: &str = "https://sourceforge.net/projects/sdcc/files/snapshot_builds/aarch64-linux-gnu/sdcc-snapshot-aarch64-linux-gnu-20260728-16725.tar.bz2/download";
const LINUX_ARM64_SDCC_SHA256: &str =
    "0cc1c7460007653efda17ccc479139fba0a2ae8f15f5d34553a4b377a336a85c";

const SEVEN_ZIP_VERSION: &str = "26.02";
const SEVEN_ZIP_BOOTSTRAP_URL: &str = "https://www.7-zip.org/a/7zr.exe";
const SEVEN_ZIP_BOOTSTRAP_SHA256: &str =
    "56b8cc9f4971cef253644fafe54063ed7fdca551d4dee0f8c6baa81b855acd72";
const SEVEN_ZIP_URL: &str = "https://www.7-zip.org/a/7z2602-x64.exe";
const SEVEN_ZIP_SHA256: &str = "6745fa76dc2ea031596d8678f6f6b99c3c1b435b4164a63485adbbc7b8d82ef0";

#[derive(Clone, Copy)]
enum ArchiveKind {
    WindowsInstaller,
    TarBz2,
}

struct SdccDistribution {
    platform: &'static str,
    version: &'static str,
    channel: &'static str,
    revision: Option<u32>,
    url: &'static str,
    sha256: &'static str,
    archive_kind: ArchiveKind,
}

pub fn run(force: bool) -> Result<()> {
    let distribution = distribution()?;
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

    let install_result = install_sdcc(&distribution, &workspace, &staging).and_then(|_| {
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
    write_manifest(&sdk_root, &distribution)?;
    println!("Installed kpdk SDK {SDK_VERSION} at {}", sdk_root.display());
    println!("Note: free-pdk includes and easypdkprog are not installed by this preview yet.");
    Ok(())
}

fn distribution() -> Result<SdccDistribution> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => Ok(SdccDistribution {
            platform: "windows-x64",
            version: "4.6.0",
            channel: "stable",
            revision: None,
            url: WINDOWS_SDCC_URL,
            sha256: WINDOWS_SDCC_SHA256,
            archive_kind: ArchiveKind::WindowsInstaller,
        }),
        ("linux", "x86_64") => Ok(SdccDistribution {
            platform: "linux-x64",
            version: "4.6.0",
            channel: "stable",
            revision: None,
            url: LINUX_X64_SDCC_URL,
            sha256: LINUX_X64_SDCC_SHA256,
            archive_kind: ArchiveKind::TarBz2,
        }),
        ("linux", "aarch64") => Ok(SdccDistribution {
            platform: "linux-arm64",
            version: "4.6.2",
            channel: "snapshot",
            revision: Some(16725),
            url: LINUX_ARM64_SDCC_URL,
            sha256: LINUX_ARM64_SDCC_SHA256,
            archive_kind: ArchiveKind::TarBz2,
        }),
        (os, arch) => Err(Error::Message(format!(
            "`kpdk sdk install` does not support {os}/{arch} yet; supported targets are Windows x64, Linux x64, and Linux ARM64"
        ))),
    }
}

fn install_sdcc(distribution: &SdccDistribution, workspace: &Path, staging: &Path) -> Result<()> {
    match distribution.archive_kind {
        ArchiveKind::WindowsInstaller => install_windows_sdcc(distribution, workspace, staging),
        ArchiveKind::TarBz2 => install_linux_sdcc(distribution, workspace, staging),
    }
}

fn install_windows_sdcc(
    distribution: &SdccDistribution,
    workspace: &Path,
    staging: &Path,
) -> Result<()> {
    let seven_zip_bootstrap = workspace.join("7zr.exe");
    let seven_zip_installer = workspace.join("7z-x64.exe");
    let seven_zip_root = workspace.join("7zip");
    let sdcc_installer = workspace.join(format!("sdcc-{}-x64-setup.exe", distribution.version));

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
    extract_with_7zip(&seven_zip_bootstrap, &seven_zip_installer, &seven_zip_root)?;
    let seven_zip = executable(&seven_zip_root, "7z");
    let seven_zip_dll = seven_zip_root.join("7z.dll");
    if !seven_zip.is_file() || !seven_zip_dll.is_file() {
        return Err(Error::Message(
            "7-Zip bootstrap did not produce `7z.exe` and `7z.dll`".into(),
        ));
    }

    println!("Downloading SDCC {}...", distribution.version);
    download_checked(distribution.url, &sdcc_installer, distribution.sha256)?;
    println!("Extracting relocatable SDCC into {}...", staging.display());
    extract_with_7zip(&seven_zip, &sdcc_installer, staging)?;
    require_sdcc(staging)
}

fn install_linux_sdcc(
    distribution: &SdccDistribution,
    workspace: &Path,
    staging: &Path,
) -> Result<()> {
    let archive_path = workspace.join("sdcc.tar.bz2");
    println!(
        "Downloading SDCC {} ({})...",
        distribution.version, distribution.channel
    );
    download_checked(distribution.url, &archive_path, distribution.sha256)?;
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

fn extract_with_7zip(seven_zip: &Path, archive: &Path, destination: &Path) -> Result<()> {
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

fn write_manifest(root: &Path, distribution: &SdccDistribution) -> Result<()> {
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
    if matches!(distribution.archive_kind, ArchiveKind::WindowsInstaller) {
        contents.push_str(&format!(
            "\n[seven_zip]\nversion = {SEVEN_ZIP_VERSION:?}\nbootstrap_source = {SEVEN_ZIP_BOOTSTRAP_URL:?}\nbootstrap_sha256 = {SEVEN_ZIP_BOOTSTRAP_SHA256:?}\nsource = {SEVEN_ZIP_URL:?}\nsha256 = {SEVEN_ZIP_SHA256:?}\n"
        ));
    }
    fs::write(&path, contents).map_err(|source| Error::Write { path, source })
}

fn executable(directory: &Path, name: &str) -> PathBuf {
    directory.join(if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    })
}
