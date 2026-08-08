# kpdk

`kpdk` is a friendly, cross-platform project and build frontend for the
[free-pdk](https://github.com/free-pdk) Padauk toolchain.

The initial release creates projects, invokes SDCC without requiring GNU Make,
checks the host toolchain, and integrates with Easy PDK Programmer.

> This repository is at an early prototype stage. SDK installation currently
> covers SDCC and the free-pdk headers on Windows x64 and Linux x64/ARM64.
> Easy PDK Programmer is shipped alongside the `kpdk` executable in release
> bundles.

## Install kpdk

Install the complete release bundle. The installer keeps `kpdk`,
`easypdkprog`, and `easypdkprog-LICENSE` together in one directory, verifies
the release SHA-256 checksum, and adds that directory to the user `PATH`.

Linux x64/ARM64:

```bash
curl -fsSL https://raw.githubusercontent.com/Markiewic/kpdk/main/scripts/install-kpdk.sh | bash
```

Windows x64 PowerShell:

```powershell
irm https://raw.githubusercontent.com/Markiewic/kpdk/main/scripts/install-kpdk.ps1 | iex
```

The default installation directories are `~/.local/bin` on Linux and
`%LOCALAPPDATA%\Programs\kpdk\bin` on Windows. Set `KPDK_INSTALL_DIR`
before running the installer to choose another directory.

For a manual installation, extract the entire release archive into a directory
already present in `PATH`. Do not copy only the `kpdk` executable:
`easypdkprog` and its license are required parts of the distribution.

## Quick start

```bash
kpdk sdk install
kpdk new blink --device PFS154
cd blink
kpdk doctor
kpdk build --release
kpdk flash
```

`kpdk sdk install` installs a relocatable SDCC toolchain, `pdk-includes`,
`easy-pdk-includes`, and reusable helper headers from `free-pdk-examples` into
the user's local application data directory.
Every downloaded file is verified against a pinned SHA-256 checksum. The header
packages are pinned to exact upstream Git commits and recorded in
`manifest.toml`.

- Windows x64 uses the official SDCC 4.6.0 installer and a pinned portable
  7-Zip bootstrap. The installers are extracted without running them, requiring
  administrator rights, or changing the registry.
- Linux x64 uses the official stable SDCC 4.6.0 binary archive.
- Linux ARM64 uses the pinned official SDCC 4.6.2 snapshot, revision 16725,
  because SDCC does not currently publish a stable ARM64 binary archive.
- All platforms receive pinned snapshots of
  [pdk-includes](https://github.com/free-pdk/pdk-includes) and
  [easy-pdk-includes](https://github.com/free-pdk/easy-pdk-includes).
- `auto_sysclock.h`, `delay.h`, `millis.h`, `serial.h`, and `startup.h` are
  downloaded directly from a pinned
  [free-pdk-examples](https://github.com/free-pdk/free-pdk-examples) commit and
  installed at the SDK include root, preserving the include style used by the
  upstream examples (for example, `#include "delay.h"`). The upstream examples
  repository currently has no repository-wide license, so kpdk does not vendor
  these helper sources.
- Windows and Linux release archives also contain Easy PDK Programmer 1.3 next
  to `kpdk`. Linux x64/ARM64 binaries are built from the pinned upstream tag;
  Windows uses the official upstream release binary verified by SHA-256.
  `kpdk probe`, `kpdk flash`, and `kpdk doctor` use this copy directly.

Archives are unpacked inside `kpdk`; no system `tar`, `bzip2`, `gzip`, `unzip`,
package manager, C compiler, or administrator rights are required. The
separately bundled `easypdkprog` executable remains licensed under GPL-3.0 by
its upstream project.

## Toolchain discovery

Use either tools available in `PATH`, or point `kpdk` at an SDK directory.

Linux and macOS:

```bash
export KPDK_SDK_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/kpdk/toolchains/2026.1"
```

Windows PowerShell:

```powershell
$env:KPDK_SDK_DIR = "$env:LOCALAPPDATA\kpdk\toolchains\2026.1"
```

The directory layout is:

```text
toolchains/2026.1/
├── sdcc/
│   ├── bin/
│   │   ├── sdcc
│   │   └── makebin
│   ├── include/
│   └── lib/
└── include/
    ├── pdk/
    │   ├── device.h
    │   └── device/
    ├── easy-pdk/
    │   ├── calibrate.h
    │   └── serial_num.h
    ├── auto_sysclock.h
    ├── delay.h
    ├── millis.h
    ├── serial.h
    └── startup.h
```

Executable names have an `.exe` suffix on Windows. Easy PDK Programmer is
discovered independently from the SDK: `KPDK_EASYPDKPROG` takes precedence,
then a copy next to the running `kpdk` executable, then `easypdkprog` from
`PATH`. If SDCC is already in `PATH`, set only `KPDK_INCLUDE_DIR` to the
free-pdk include directory.

## Commands

```text
kpdk sdk install [--force]
kpdk new <name> --device <device>
kpdk build [--release]
kpdk clean
kpdk doctor
kpdk probe
kpdk flash [--port COM5]
kpdk vscode [--project <path>]
```

`kpdk flash` asks for explicit confirmation before writing an OTP `PMS` device.

## VS Code

`kpdk new` creates ready-to-use VS Code integration from `pdk.toml` and the
discovered SDK. Open the project folder and accept the recommended Microsoft
C/C++ extension; free-pdk headers, completion, navigation, and build diagnostics
work without configuring include paths by hand.

Generated integration includes:

- `.vscode/c_cpp_properties.json` with the SDK include path, selected device,
  `F_CPU`, and `TARGET_VDD_MV`;
- `.vscode/kpdk-intellisense.h` to make SDCC-only storage classes and inline
  assembly understandable to the Microsoft C/C++ parser without affecting builds;
- `.vscode/tasks.json` with build, release build, flash, and refresh tasks plus
  an SDCC problem matcher;
- `.vscode/extensions.json` recommending `ms-vscode.cpptools`;
- `compile_commands.json` describing the SDCC compilation arguments for other
  editor tooling.

The machine-specific `c_cpp_properties.json` and `compile_commands.json` are
gitignored. After cloning a project, changing the MCU/clock/voltage, or switching
SDK locations, regenerate them with:

```bash
kpdk vscode
```

Microsoft C/C++ uses a GCC/Clang-compatible parser rather than SDCC itself, so
`kpdk build` remains authoritative for compiler diagnostics.

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The CI integration test installs the complete SDK, verifies it with
`kpdk doctor`, creates a fresh PFS154 project, and compiles it on Windows x64,
Linux x64, and Linux ARM64.

## Roadmap

- Linux udev setup and programmer diagnostics
- macOS SDK bundle
- WinGet and Scoop packages
- Complete, generated device database
