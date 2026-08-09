use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::ProjectFile;
use crate::device;
use crate::error::{Error, Result};
use crate::process;
use crate::toolchain::Toolchain;

pub struct Artifacts {
    pub ihx: PathBuf,
    pub bin: PathBuf,
}

pub fn run(root: &Path, release: bool) -> Result<Artifacts> {
    let config = ProjectFile::load(root)?;
    let architecture = device::architecture(&config.project.device)?;
    let toolchain = Toolchain::discover();
    let build_dir = root.join("build");
    fs::create_dir_all(&build_dir).map_err(|source| Error::Write {
        path: build_dir.clone(),
        source,
    })?;

    let mut objects = Vec::new();
    for source in &config.build.sources {
        let source_path = root.join(source);
        if !source_path.is_file() {
            return Err(Error::Message(format!(
                "source file `{}` does not exist",
                source_path.display()
            )));
        }
        let stem = source_path
            .file_stem()
            .ok_or_else(|| Error::Message(format!("invalid source path `{}`", source.display())))?;
        let object = build_dir.join(stem).with_extension("rel");

        let mut args: Vec<OsString> = vec![
            format!("-m{}", architecture.sdcc_target()).into(),
            "-c".into(),
            "--std-sdcc11".into(),
            if release {
                "--opt-code-size".into()
            } else {
                "--debug".into()
            },
            format!("-D{}", config.project.device.to_ascii_uppercase()).into(),
            format!("-DF_CPU={}", config.project.clock_hz).into(),
            format!("-DTARGET_VDD_MV={}", config.project.target_vdd_mv).into(),
        ];
        if let Some(include) = &toolchain.include {
            args.push(format!("-I{}", include.display()).into());
        }
        args.extend([
            "-o".into(),
            object.as_os_str().to_owned(),
            source_path.as_os_str().to_owned(),
        ]);
        run_sdcc(&toolchain, args)?;
        objects.push(object);
    }

    let base = build_dir.join(&config.project.name);
    let ihx = base.with_extension("ihx");
    let bin = base.with_extension("bin");
    let mut link_args: Vec<OsString> = vec![
        format!("-m{}", architecture.sdcc_target()).into(),
        "--out-fmt-ihx".into(),
        "-o".into(),
        ihx.as_os_str().to_owned(),
    ];
    link_args.extend(objects.iter().map(|path| path.as_os_str().to_owned()));
    run_sdcc(&toolchain, link_args)?;
    process::run(
        &toolchain.makebin,
        vec![
            OsString::from("-p"),
            ihx.as_os_str().to_owned(),
            bin.as_os_str().to_owned(),
        ],
    )?;

    Ok(Artifacts { ihx, bin })
}

pub fn print_artifacts(artifacts: &Artifacts) {
    println!("Built {}", artifacts.ihx.display());
    println!("Built {}", artifacts.bin.display());
}

fn run_sdcc(toolchain: &Toolchain, args: Vec<OsString>) -> Result<()> {
    if let Some(compiler_path) = &toolchain.sdcc_compiler_path {
        let envs = [(
            OsString::from("COMPILER_PATH"),
            compiler_path.as_os_str().to_owned(),
        )];
        process::run_with_env(&toolchain.sdcc, args, &envs)
    } else {
        process::run(&toolchain.sdcc, args)
    }
}
