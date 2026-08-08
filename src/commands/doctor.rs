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

    let include_checks = [
        ("pdk-includes", "pdk/device.h"),
        ("easy-pdk", "easy-pdk/calibrate.h"),
        ("auto-sysclock", "auto_sysclock.h"),
        ("delay", "delay.h"),
        ("millis", "millis.h"),
        ("serial", "serial.h"),
        ("startup", "startup.h"),
    ];
    match tools.include {
        Some(path) => {
            for (label, relative) in include_checks {
                if path.join(relative).is_file() {
                    println!("{label:<16} OK");
                } else {
                    println!("{label:<16} MISSING ({})", path.display());
                    failed = true;
                }
            }
        }
        None => {
            for (label, _) in include_checks {
                println!("{label:<16} NOT CONFIGURED");
            }
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
