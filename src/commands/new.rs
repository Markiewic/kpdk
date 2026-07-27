use std::fs;
use std::path::Path;

use crate::device;
use crate::error::{Error, Result};

pub fn run(name: &str, device_name: &str, clock: u32, vdd: u16) -> Result<()> {
    device::architecture(device_name)?;
    let root = Path::new(name);
    if root.exists() {
        return Err(Error::Message(format!(
            "destination `{}` already exists",
            root.display()
        )));
    }

    fs::create_dir_all(root.join("src")).map_err(|source| Error::Write {
        path: root.to_owned(),
        source,
    })?;

    write(
        &root.join("pdk.toml"),
        &format!(
            "[project]\nname = {name:?}\ndevice = {device:?}\nclock_hz = {clock}\ntarget_vdd_mv = {vdd}\n\n[build]\nsources = [\"src/main.c\"]\n\n[programmer]\nport = \"auto\"\n",
            device = device_name.to_ascii_uppercase()
        ),
    )?;
    write(&root.join("src/main.c"), MAIN_C)?;
    write(&root.join(".gitignore"), "/build/\n")?;

    println!("Created `{name}` for {}", device_name.to_ascii_uppercase());
    println!("  cd {name}");
    println!("  kpdk build");
    Ok(())
}

fn write(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).map_err(|source| Error::Write {
        path: path.to_owned(),
        source,
    })
}

const MAIN_C: &str = r#"#include <pdk/device.h>
#include <pdk/sysclock.h>

unsigned char _sdcc_external_startup(void)
{
    /* Configure the clock explicitly for your device before using timing code. */
    return 0;
}

void main(void)
{
    while (1) {
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_project() {
        let previous = std::env::current_dir().unwrap();
        let temp = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        run("firmware", "PFS154", 8_000_000, 5_000).unwrap();
        assert!(temp.path().join("firmware/pdk.toml").is_file());
        assert!(temp.path().join("firmware/src/main.c").is_file());

        std::env::set_current_dir(previous).unwrap();
    }
}
