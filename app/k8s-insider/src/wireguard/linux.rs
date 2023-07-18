use std::{path::Path, process::Command};

use anyhow::anyhow;

pub fn wg_quick_up(config_path: &Path) -> anyhow::Result<()> {
    let command_result = Command::new("wg-quick")
        .arg("up")
        .arg(config_path)
        .status()?;

    if !command_result.success() {
        return Err(anyhow!("An error occurred when setting up the tunnel interface!"));
    }

    Ok(())
}