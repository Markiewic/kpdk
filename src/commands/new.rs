use std::fs;
use std::path::Path;

use crate::commands::vscode;
use crate::device;
use crate::error::{Error, Result};

pub fn run(name: &str, device_name: &str, clock: u32, vdd: u16) -> Result<()> {
    let architecture = device::architecture(device_name)?;
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
    write(
        &root.join("README.md"),
        &project_readme(name, device_name, clock, vdd),
    )?;
    write(
        &root.join("AGENTS.md"),
        &project_agents(device_name, architecture.sdcc_target(), clock, vdd),
    )?;
    write(
        &root.join(".gitignore"),
        "/build/\n/compile_commands.json\n",
    )?;
    vscode::run(root)?;

    println!("Created `{name}` for {}", device_name.to_ascii_uppercase());
    println!("  cd {name}");
    println!("  kpdk build");
    Ok(())
}

fn project_agents(device_name: &str, architecture: &str, clock: u32, vdd: u16) -> String {
    format!(
        r#"# AGENTS.md

## Project

- Target device: `{device}`
- SDCC architecture: `{architecture}`
- Clock: `{clock} Hz`
- Target VDD: `{vdd} mV`
- Firmware entry point: `src/main.c`
- Project configuration: `pdk.toml`

## Commands

Install and verify the toolchain before the first build:

```sh
kpdk sdk install
kpdk doctor
```

Build and validate firmware with:

```sh
kpdk build
kpdk build --release
```

Run `kpdk vscode` after changing the MCU, clock, VDD, SDK location, or editor configuration.

## Firmware rules

- Build with `kpdk`; do not substitute GCC or Clang for SDCC.
- Treat diagnostics from `kpdk build` and SDCC as authoritative.
- Use register names, macros, and headers provided by the installed Padauk SDK.
- Do not invent Arduino APIs, undocumented registers, device aliases, or header names.
- Do not change the target MCU, clock, VDD, pin assignments, or signal polarity without explicit confirmation.
- Keep hardware-specific assumptions visible in code comments and task summaries.
- Run `kpdk build` after every firmware change and report any remaining warnings.
- Do not edit generated files in `build/` or `compile_commands.json` by hand.

## Reference implementations

Before writing peripheral code, inspect the pinned `free-pdk-examples` snapshot
installed with the SDK. Start with `examples/index.toml`, then read the relevant
example under `examples/upstream/`:

- Windows: `%LOCALAPPDATA%\\kpdk\\toolchains\\2026.1\\examples`
- Linux/macOS: `$HOME/.local/share/kpdk/toolchains/2026.1/examples`
- Custom SDK: `$KPDK_SDK_DIR/examples`

Use these examples as references for GPIO, clocks, timers, interrupts, PWM,
sleep, and software serial. They are upstream examples, not project templates:
adapt them to the exact target MCU and current SDK headers. Never assume that an
example's registers, peripherals, pins, clock, voltage, or polarity are valid for
this project. Build the adapted code with `kpdk build`.

Upstream source: https://github.com/free-pdk/free-pdk-examples

## Hardware safety

- A successful build verifies compilation only; it does not prove behavior on physical hardware.
- Never run `kpdk flash` unless the user explicitly requests physical programming.
- Before flashing an M-series OTP device, warn that it cannot be erased or reprogrammed.
- Flag oscillator-calibration placeholders and state the calibration voltage.
- For LEDs, use a current-limiting resistor and state whether the GPIO sources or sinks current.
"#,
        device = device_name.to_ascii_uppercase()
    )
}

fn project_readme(name: &str, device_name: &str, clock: u32, vdd: u16) -> String {
    format!(
        r#"# {name}

Padauk firmware project for **{device}**, created with [kpdk](https://github.com/Markiewic/kpdk).

## Getting started

### 1. Install kpdk

Windows x64 (PowerShell):

```powershell
irm https://raw.githubusercontent.com/Markiewic/kpdk/main/scripts/install-kpdk.ps1 | iex
```

Linux x64/ARM64:

```bash
curl -fsSL https://raw.githubusercontent.com/Markiewic/kpdk/main/scripts/install-kpdk.sh | bash
```

You can also download the complete bundle from the [latest kpdk release](https://github.com/Markiewic/kpdk/releases/latest).

### 2. Install the SDK

```bash
kpdk sdk install
kpdk doctor
```

This installs the SDCC toolchain and the free-pdk headers used by the project.

### 3. Build

From the project directory:

```bash
kpdk build
```

For an optimized release build:

```bash
kpdk build --release
```

Build artifacts are written to `build/`.

### 4. Program the device

With an Easy PDK Programmer connected:

```bash
kpdk probe
kpdk flash
```

Some Padauk devices are OTP and cannot be erased after programming. `kpdk flash` asks for confirmation before writing an OTP target.

## Writing firmware

The firmware is C compiled with SDCC. Start in:

```text
src/main.c
```

If you add more C source files, list them under `build.sources` in `pdk.toml`.

This project was created with:

- device: `{device}`
- clock: `{clock} Hz`
- target VDD: `{vdd} mV`

The project settings live in `pdk.toml`.

## Editor

[Visual Studio Code](https://code.visualstudio.com/) with the recommended
[Microsoft C/C++ extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode.cpptools)
is the easiest way to get completion, navigation, and build diagnostics.

The repository includes portable VS Code configuration in `.vscode/`, so a fresh clone is ready to open. Run:

```bash
kpdk vscode
```

after changing the MCU, clock, VDD, SDK location, or when you want to refresh the editor configuration and `compile_commands.json`.

For compiler errors, `kpdk build` is authoritative because the firmware is compiled by SDCC.
"#,
        device = device_name.to_ascii_uppercase()
    )
}

fn write(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).map_err(|source| Error::Write {
        path: path.to_owned(),
        source,
    })
}

const MAIN_C: &str = r#"#include <pdk/device.h>
#include <pdk/sysclock.h>

unsigned char __sdcc_external_startup(void)
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
        let readme = fs::read_to_string(temp.path().join("firmware/README.md")).unwrap();
        assert!(readme.contains("# firmware"));
        assert!(readme.contains("**PFS154**"));
        assert!(readme.contains("kpdk sdk install"));
        assert!(readme.contains("kpdk build --release"));
        assert!(readme.contains("src/main.c"));
        assert!(readme.contains("Visual Studio Code"));
        let agents = fs::read_to_string(temp.path().join("firmware/AGENTS.md")).unwrap();
        assert!(agents.contains("Target device: `PFS154`"));
        assert!(agents.contains("SDCC architecture: `pdk14`"));
        assert!(agents.contains("Clock: `8000000 Hz`"));
        assert!(agents.contains("Target VDD: `5000 mV`"));
        assert!(agents.contains("kpdk build --release"));
        assert!(agents.contains("examples/index.toml"));
        assert!(agents.contains("free-pdk-examples"));
        assert!(agents.contains("Never run `kpdk flash`"));
        assert!(agents.contains("OTP device"));
        let main_c = fs::read_to_string(temp.path().join("firmware/src/main.c")).unwrap();
        assert!(main_c.contains("unsigned char __sdcc_external_startup(void)"));
        assert!(temp
            .path()
            .join("firmware/.vscode/c_cpp_properties.json")
            .is_file());
        assert!(temp.path().join("firmware/.vscode/tasks.json").is_file());
        assert!(temp
            .path()
            .join("firmware/.vscode/extensions.json")
            .is_file());
        assert!(temp
            .path()
            .join("firmware/.vscode/kpdk-intellisense.h")
            .is_file());
        assert!(temp.path().join("firmware/compile_commands.json").is_file());

        let gitignore = fs::read_to_string(temp.path().join("firmware/.gitignore")).unwrap();
        assert!(!gitignore.contains("c_cpp_properties.json"));

        let cpp_properties =
            fs::read_to_string(temp.path().join("firmware/.vscode/c_cpp_properties.json")).unwrap();
        assert!(cpp_properties.contains("\"name\": \"Win32\""));
        assert!(cpp_properties.contains("\"name\": \"Linux\""));
        assert!(cpp_properties.contains("\"name\": \"Mac\""));
        assert!(cpp_properties.contains("${env:LOCALAPPDATA}"));
        assert!(cpp_properties.contains("${env:HOME}"));
        assert!(cpp_properties.contains("${default}"));

        std::env::set_current_dir(previous).unwrap();
    }
}
