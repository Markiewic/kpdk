# kpdk

`kpdk` is a friendly, cross-platform project and build frontend for the
[free-pdk](https://github.com/free-pdk) Padauk toolchain.

The initial release creates projects, invokes SDCC without requiring GNU Make,
checks the host toolchain, and integrates with Easy PDK Programmer.

> This repository is at an early prototype stage. SDK installation currently
> covers SDCC on Windows x64 and Linux x64/ARM64; the remaining free-pdk tools
> are the next milestone.

## Quick start

```bash
kpdk sdk install
kpdk new blink --device PFS154
cd blink
kpdk doctor
kpdk build --release
kpdk flash
```

`kpdk sdk install` installs a relocatable SDCC toolchain into the user's local
application data directory. Every downloaded file is verified against a pinned
SHA-256 checksum.

- Windows x64 uses the official SDCC 4.6.0 installer and a pinned portable
  7-Zip bootstrap. The installers are extracted without running them, requiring
  administrator rights, or changing the registry.
- Linux x64 uses the official stable SDCC 4.6.0 binary archive.
- Linux ARM64 uses the pinned official SDCC 4.6.2 snapshot, revision 16725,
  because SDCC does not currently publish a stable ARM64 binary archive.

Linux archives are unpacked inside `kpdk`; no system `tar`, `bzip2`,
`unzip`, package manager, or administrator rights are required. The preview
does not yet install `pdk-includes`, `easy-pdk-includes`, or
`easypdkprog`.

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
    └── easy-pdk/
```

Executable names have an `.exe` suffix on Windows. If SDCC and Easy PDK
Programmer are already in `PATH`, set only `KPDK_INCLUDE_DIR` to the
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
```

`kpdk flash` asks for explicit confirmation before writing an OTP `PMS` device.

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Roadmap

- Install free-pdk includes and Easy PDK Programmer through `kpdk sdk install`
- macOS SDK bundle
- WinGet and Scoop packages
- Complete, generated device database
- VS Code configuration and examples
