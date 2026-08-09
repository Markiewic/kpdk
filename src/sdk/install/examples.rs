use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use tar::Archive;

use crate::error::{Error, Result};

use super::{download_checked, recreate_directory};

const COMMIT: &str = "019c6204fef80bae6230ad299bcdf03f0f8a798f";
const URL: &str = "https://codeload.github.com/free-pdk/free-pdk-examples/tar.gz/019c6204fef80bae6230ad299bcdf03f0f8a798f";
const SHA256: &str = "79aaaa05f4343019b86994363ac447477f8eabf81c833c8fee3f662ab34496f0";

pub(super) fn install(workspace: &Path, staging: &Path) -> Result<()> {
    let archive_path = workspace.join("free-pdk-examples.tar.gz");
    let unpack_root = workspace.join("free-pdk-examples-unpacked");

    println!("Downloading free-pdk-examples at {COMMIT}...");
    download_checked(URL, &archive_path, SHA256)?;
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
    let upstream = staging.join("upstream");
    fs::rename(source, &upstream).map_err(|source| Error::Write {
        path: upstream,
        source,
    })?;
    let index = staging.join("index.toml");
    fs::write(&index, INDEX).map_err(|source| Error::Write {
        path: index,
        source,
    })?;
    verify(staging)
}

pub(super) fn verify(root: &Path) -> Result<()> {
    let required = [
        root.join("index.toml"),
        root.join("upstream/README.md"),
        root.join("upstream/BlinkLED/main.c"),
        root.join("upstream/FadeLED/main.c"),
        root.join("upstream/Serial_HelloWorld/main.c"),
    ];
    if let Some(path) = required.iter().find(|path| !path.is_file()) {
        Err(Error::Message(format!(
            "free-pdk example installation is missing `{}`",
            path.display()
        )))
    } else {
        Ok(())
    }
}

pub(super) fn append_manifest(contents: &mut String) {
    contents.push_str(&format!(
        "\n[free_pdk_examples]\ncommit = {COMMIT:?}\nsource = {URL:?}\nsha256 = {SHA256:?}\nindex = \"examples/index.toml\"\n"
    ));
}

fn single_directory(root: &Path) -> Result<PathBuf> {
    let directories = fs::read_dir(root)
        .map_err(|source| Error::Read {
            path: root.to_owned(),
            source,
        })?
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|source| Error::Read {
            path: root.to_owned(),
            source,
        })?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    if directories.len() == 1 {
        Ok(directories[0].clone())
    } else {
        Err(Error::Message(format!(
            "free-pdk-examples archive extracted into {} top-level directories instead of one",
            directories.len()
        )))
    }
}

const INDEX: &str = r#"source = "https://github.com/free-pdk/free-pdk-examples"
commit = "019c6204fef80bae6230ad299bcdf03f0f8a798f"
root = "upstream"
usage = "Reference only; verify the target MCU, registers, pins, clock, voltage, and polarity before adapting code."

[[examples]]
name = "BlinkLED"
topics = ["gpio", "delay", "led"]
default_device = "PFS154"

[[examples]]
name = "BlinkLED_WithIRQ"
topics = ["gpio", "timer", "interrupt", "millis"]
default_device = "PFS154"

[[examples]]
name = "FadeLED"
topics = ["pwm", "timer", "led"]
default_device = "PFS154"

[[examples]]
name = "ReadButton_WriteLED"
topics = ["gpio", "input", "pull-up", "led"]
default_device = "PFS154"

[[examples]]
name = "ReadButton_WriteSerial"
topics = ["gpio", "input", "serial"]
default_device = "PFS154"

[[examples]]
name = "Serial_HelloWorld"
topics = ["serial", "timing"]
default_device = "PFS154"

[[examples]]
name = "SleepWake"
topics = ["sleep", "wakeup", "gpio", "low-power"]
default_device = "PFS154"
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_names_every_upstream_example() {
        for name in [
            "BlinkLED",
            "BlinkLED_WithIRQ",
            "FadeLED",
            "ReadButton_WriteLED",
            "ReadButton_WriteSerial",
            "Serial_HelloWorld",
            "SleepWake",
        ] {
            assert!(INDEX.contains(&format!("name = {name:?}")));
        }
    }
}
