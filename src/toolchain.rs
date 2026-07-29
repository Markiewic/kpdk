use std::env;
use std::path::{Path, PathBuf};

pub struct Toolchain {
    pub sdcc: String,
    pub sdcc_compiler_path: Option<PathBuf>,
    pub makebin: String,
    pub easypdkprog: String,
    pub include: Option<PathBuf>,
}

impl Toolchain {
    pub fn discover() -> Self {
        if let Some(root) = env::var_os("KPDK_SDK_DIR")
            .map(PathBuf::from)
            .or_else(default_sdk_root)
            .filter(|root| root.is_dir())
        {
            let bin = root.join("bin");
            let sdcc_bin = root.join("sdcc").join("bin");
            return Self {
                sdcc: executable(&sdcc_bin, "sdcc"),
                sdcc_compiler_path: cfg!(windows).then_some(sdcc_bin.clone()),
                makebin: executable(&sdcc_bin, "makebin"),
                easypdkprog: executable(&bin, "easypdkprog"),
                include: Some(root.join("include")),
            };
        }

        Self {
            sdcc: "sdcc".into(),
            sdcc_compiler_path: None,
            makebin: "makebin".into(),
            easypdkprog: "easypdkprog".into(),
            include: env::var_os("KPDK_INCLUDE_DIR").map(PathBuf::from),
        }
    }
}

pub fn default_sdk_root() -> Option<PathBuf> {
    dirs::data_local_dir().map(|root| root.join("kpdk").join("toolchains").join("2026.1"))
}

fn executable(directory: &Path, name: &str) -> String {
    let extension = if cfg!(windows) { ".exe" } else { "" };
    directory
        .join(format!("{name}{extension}"))
        .to_string_lossy()
        .into_owned()
}
