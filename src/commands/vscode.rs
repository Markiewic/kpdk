use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::ProjectFile;
use crate::device;
use crate::error::{Error, Result};
use crate::toolchain::{default_sdk_root, Toolchain, TOOLCHAIN_VERSION};

pub fn run(root: &Path) -> Result<()> {
    let config = ProjectFile::load(root)?;
    let architecture = device::architecture(&config.project.device)?;
    let toolchain = Toolchain::discover();
    let include = toolchain
        .include
        .clone()
        .or_else(|| default_sdk_root().map(|path| path.join("include")))
        .ok_or_else(|| Error::Message("cannot determine the kpdk SDK include directory".into()))?;

    let vscode = root.join(".vscode");
    fs::create_dir_all(&vscode).map_err(|source| Error::Write {
        path: vscode.clone(),
        source,
    })?;

    write(
        &vscode.join("c_cpp_properties.json"),
        &cpp_properties(&config),
    )?;
    write(&vscode.join("extensions.json"), EXTENSIONS)?;
    write(&vscode.join("kpdk-intellisense.h"), INTELLISENSE_HEADER)?;
    write(&vscode.join("tasks.json"), TASKS)?;
    write(
        &root.join("compile_commands.json"),
        &compile_commands(
            root,
            &config,
            architecture.sdcc_target(),
            &toolchain,
            &include,
        ),
    )?;

    println!("Updated VS Code configuration in {}", vscode.display());
    Ok(())
}

fn cpp_properties(config: &ProjectFile) -> String {
    let configurations = [
        (
            "Win32",
            format!(
                "${{env:LOCALAPPDATA}}/kpdk/toolchains/{TOOLCHAIN_VERSION}/include"
            ),
        ),
        (
            "Linux",
            format!(
                "${{env:HOME}}/.local/share/kpdk/toolchains/{TOOLCHAIN_VERSION}/include"
            ),
        ),
        (
            "Mac",
            format!(
                "${{env:HOME}}/Library/Application Support/kpdk/toolchains/{TOOLCHAIN_VERSION}/include"
            ),
        ),
    ]
    .into_iter()
    .map(|(name, include)| cpp_configuration(config, name, &include))
    .collect::<Vec<_>>()
    .join(",\\n");

    format!("{\\n  \\\"configurations\\\": [\\n{configurations}\\n  ],\\n  \\\"version\\\": 4\\n}\\n")
}

fn cpp_configuration(config: &ProjectFile, name: &str, include: &str) -> String {
    format!(
        r#"    {{
      "name": {name},
      "includePath": [
        "${{workspaceFolder}}/src",
        {include}
      ],
      "defines": [
        {device},
        {clock},
        {vdd}
      ],
      "compilerPath": "",
      "forcedInclude": [
        "${{workspaceFolder}}/.vscode/kpdk-intellisense.h"
      ],
      "cStandard": "c11",
      "intelliSenseMode": "${{default}}"
    }}"#,
        name = json_string(name),
        include = json_string(include),
        device = json_string(&config.project.device.to_ascii_uppercase()),
        clock = json_string(&format!("F_CPU={}", config.project.clock_hz)),
        vdd = json_string(&format!("TARGET_VDD_MV={}", config.project.target_vdd_mv)),
    )
}

fn compile_commands(
    root: &Path,
    config: &ProjectFile,
    sdcc_target: &str,
    toolchain: &Toolchain,
    include: &Path,
) -> String {
    let directory = absolute_path(root);
    let mut entries = Vec::new();

    for source in &config.build.sources {
        let source_path = root.join(source);
        let source_path = absolute_path(&source_path);
        let object = absolute_path(
            &root
                .join("build")
                .join(
                    source_path
                        .file_stem()
                        .unwrap_or_else(|| OsStr::new("source")),
                )
                .with_extension("rel"),
        );
        let mut arguments = vec![
            toolchain.sdcc.clone(),
            format!("-m{sdcc_target}"),
            "-c".into(),
            "--std-sdcc11".into(),
            "--debug".into(),
            format!("-D{}", config.project.device.to_ascii_uppercase()),
            format!("-DF_CPU={}", config.project.clock_hz),
            format!("-DTARGET_VDD_MV={}", config.project.target_vdd_mv),
        ];
        arguments.push(format!("-I{}", include.display()));
        arguments.extend([
            "-o".into(),
            object.to_string_lossy().into_owned(),
            source_path.to_string_lossy().into_owned(),
        ]);
        let arguments = arguments
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(", ");
        entries.push(format!(
            "  {{\n    \"directory\": {},\n    \"file\": {},\n    \"arguments\": [{}]\n  }}",
            json_string(&directory.to_string_lossy()),
            json_string(&source_path.to_string_lossy()),
            arguments
        ));
    }

    format!("[\n{}\n]\n", entries.join(",\n"))
}

fn absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_owned())
    }
}

fn json_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                use std::fmt::Write;
                let _ = write!(output, "\\u{:04x}", character as u32);
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}

fn write(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).map_err(|source| Error::Write {
        path: path.to_owned(),
        source,
    })
}

const EXTENSIONS: &str = r#"{
  "recommendations": [
    "ms-vscode.cpptools"
  ]
}
"#;

// Microsoft C/C++ does not parse SDCC storage classes and inline assembly.
// This header is force-included by IntelliSense only; it is never passed to SDCC.
const INTELLISENSE_HEADER: &str = r#"#ifndef KPDK_INTELLISENSE_H
#define KPDK_INTELLISENSE_H

#ifndef __SDCC
#define __sfr volatile unsigned char
#define __sfr16 volatile unsigned short
#define __sfr32 volatile unsigned long
#define __sbit volatile unsigned char
#define __at(address)
#define __asm__(...)
#define __critical
#define __interrupt(...)
#define __naked
#define __code
#define __data
#define __idata
#define __xdata
#endif

#endif
"#;

const TASKS: &str = r#"{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "kpdk: build",
      "type": "process",
      "command": "kpdk",
      "args": ["build"],
      "options": { "cwd": "${workspaceFolder}" },
      "problemMatcher": {
        "owner": "cpp",
        "fileLocation": ["relative", "${workspaceFolder}"],
        "pattern": {
          "regexp": "^(.*):(\\d+):\\s+(warning|error)(?:\\s+\\d+)?:\\s+(.*)$",
          "file": 1,
          "line": 2,
          "severity": 3,
          "message": 4
        }
      },
      "group": { "kind": "build", "isDefault": true }
    },
    {
      "label": "kpdk: build (release)",
      "type": "process",
      "command": "kpdk",
      "args": ["build", "--release"],
      "options": { "cwd": "${workspaceFolder}" },
      "problemMatcher": {
        "owner": "cpp",
        "fileLocation": ["relative", "${workspaceFolder}"],
        "pattern": {
          "regexp": "^(.*):(\\d+):\\s+(warning|error)(?:\\s+\\d+)?:\\s+(.*)$",
          "file": 1,
          "line": 2,
          "severity": 3,
          "message": 4
        }
      }
    },
    {
      "label": "kpdk: flash",
      "type": "process",
      "command": "kpdk",
      "args": ["flash"],
      "options": { "cwd": "${workspaceFolder}" },
      "problemMatcher": [],
      "presentation": { "reveal": "always", "panel": "dedicated" }
    },
    {
      "label": "kpdk: refresh VS Code",
      "type": "process",
      "command": "kpdk",
      "args": ["vscode"],
      "options": { "cwd": "${workspaceFolder}" },
      "problemMatcher": []
    }
  ]
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_windows_paths_for_json() {
        assert_eq!(
            json_string(r#"C:\Users\Test\"sdk"#),
            r#""C:\\Users\\Test\\\"sdk""#
        );
    }
}
