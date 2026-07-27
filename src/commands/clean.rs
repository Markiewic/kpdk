use std::fs;
use std::path::Path;

use crate::error::{Error, Result};

pub fn run(root: &Path) -> Result<()> {
    let build = root.join("build");
    if build.is_dir() {
        fs::remove_dir_all(&build).map_err(|source| Error::Write {
            path: build.clone(),
            source,
        })?;
        println!("Removed {}", build.display());
    }
    Ok(())
}
