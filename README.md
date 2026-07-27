# kpdk

`kpdk` is a friendly, cross-platform project and build frontend for the
[free-pdk](https://github.com/free-pdk) Padauk toolchain.

The initial release creates projects, invokes SDCC without requiring GNU Make,
checks the host toolchain, and integrates with Easy PDK Programmer.

> This repository is at an early prototype stage. Automatic SDK installation is
> the next milestone; the current version expects an installed toolchain.

## Quick start

```powershell
kpdk sdk install
kpdk new blink --device PFS154
cd blink
kpdk doctor
kpdk build --release
kpdk flash
```

`kpdk sdk install` currently installs the official relocatable SDCC 4.6.0
distribution on Windows x64. The preview does not yet install `pdk-includes`,
`easy-pdk-includes`, or `easypdkprog`.

## Toolchain discovery

Use either tools available in `PATH`, or point `kpdk` at an SDK directory:

```powershell
$env:KPDK_SDK_DIR = "$env:LOCALAPPDATA\kpdk\toolchains\2026.1"
```

The directory layout is:

```text
toolchains/2026.1/
├── sdcc/
│   ├── bin/
│   │   ├── sdcc.exe
│   │   └── makebin.exe
│   ├── include/
│   └── lib/
├── bin/
│   └── easypdkprog.exe
└── include/
    ├── pdk/
    └── easy-pdk/
```

If SDCC and Easy PDK Programmer are already in `PATH`, set only:

```powershell
$env:KPDK_INCLUDE_DIR = "C:\free-pdk\include"
```

## Commands

```text
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

- Versioned `kpdk sdk install`
- Windows x64 SDK bundle
- WinGet and Scoop packages
- Linux and macOS SDK bundles
- Complete, generated device database
- VS Code configuration and examples
