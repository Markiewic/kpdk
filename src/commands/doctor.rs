use crate::process;
use crate::toolchain::Toolchain;

use crate::error::{Error, Result};

pub fn run() -> Result<()> {
    let tools = Toolchain::discover();
    let checks = [
        ("sdcc", tools.sdcc.as_str(), "-v"),
        ("makebin", tools.makebin.as_str(), "-h"),
        ("easypdkprog", tools.easypdkprog.as_str(), "--version"),
    ];
    let mut failed = false;

    for (label, program, argument) in checks {
        let ok = process::version(program, argument);
        println!("{label:<16} {}", if ok { "OK" } else { "NOT FOUND" });
        failed |= !ok;
    }
    match tools.include {
        Some(path) if path.join("pdk/device.h").is_file() => {
            println!("{:<16} OK", "pdk-includes")
        }
        Some(path) => {
            println!("{:<16} MISSING ({})", "pdk-includes", path.display());
            failed = true;
        }
        None => {
            println!("{:<16} NOT CONFIGURED", "pdk-includes");
            failed = true;
        }
    }

    if failed {
        Err(Error::Message(
            "toolchain is incomplete; set KPDK_SDK_DIR or KPDK_INCLUDE_DIR".into(),
        ))
    } else {
        Ok(())
    }
}
