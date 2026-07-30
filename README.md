# kpdk

`kpdk` is a friendly, cross-platform project and build frontend for the
[free-pdk](https://github.com/free-pdk) Padauk toolchain.

The initial release creates projects, invokes SDCC without requiring GNU Make,
checks the host toolchain, and integrates with Easy PDK Programmer.

> This repository is at an early prototype stage. SDK installation currently
> covers SDCC, the free-pdk headers, and Easy PDK Programmer on Windows x64 and
> Linux x64/ARM64.

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
`easy-pdk-includes`, and `easypdkprog` into the user's local application data
directory. Every
downloaded file is verified against a pinned SHA-256 checksum. The header
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
- Windows and Linux release archives also contain Easy PDK Programmer 1.3.
  Linux x64/ARM64 binaries are built from the pinned upstream tag; Windows uses
  the official upstream release binary verified by SHA-256. `sdk install`
  copies and verifies the bundled executable without requiring a C compiler.

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
├── bin/
│   └── easypdkprog
└── include/
    ├── pdk/
    │   ├── device.h
    │   └── device/
    └── easy-pdk/
        ├── calibrate.h
        └── serial_num.h
```

Executable names have an `.exe` suffix on Windows. If SDCC and Easy PDK
Programmer are already in `PATH`, set only `KPDK_INCLUDE_DIR` to the
free-pdk include directory. Source builds and custom packages can set
`KPDK_EASYPDKPROG` to an existing programmer executable before running
`kpdk sdk install`.

## Commands

```text
kpdk sdk install [--force]
kpdk new <name> --device <device>
kpdk build [--release]
kpdk clean
kpdk doctor
kpdk probe
kpdk flash [--port COM5]
```

`kpdk flash` asks for explicit confirmation before writing an OTP `PMS` device.

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
- VS Code configuration and examples
