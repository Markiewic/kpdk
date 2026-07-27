use std::env;
use std::path::{Path, PathBuf};

pub struct Toolchain {
    pub sdcc: String,
    pub makebin: String,
    pub easypdkprog: String,
    pub include: Option<PathBuf>,
}

impl Toolchain {
    pub fn discover() -> Self {
        if let Some(root) = env::var_os("KPDK_SDK_DIR").map(PathBuf::from) {
            let bin = root.join("bin");
            return Self {
                sdcc: executable(&bin, "sdcc"),
                makebin: executable(&bin, "makebin"),
                easypdkprog: executable(&bin, "easypdkprog"),
                include: Some(root.join("include")),
            };
        }

        Self {
            sdcc: "sdcc".into(),
            makebin: "makebin".into(),
            easypdkprog: "easypdkprog".into(),
            include: env::var_os("KPDK_INCLUDE_DIR").map(PathBuf::from),
        }
    }
}

fn executable(directory: &Path, name: &str) -> String {
    let extension = if cfg!(windows) { ".exe" } else { "" };
    directory
        .join(format!("{name}{extension}"))
        .to_string_lossy()
        .into_owned()
}
