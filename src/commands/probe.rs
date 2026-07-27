use crate::error::Result;
use crate::process;
use crate::toolchain::Toolchain;

pub fn run() -> Result<()> {
    process::run(&Toolchain::discover().easypdkprog, ["probe"])
}
