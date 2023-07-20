use std::{path::Path, process::Command};

use anyhow::anyhow;
use log::warn;

pub fn chmod(path: &Path, permissions: u16) -> anyhow::Result<()> {
    if permissions > 0o777 {
        return Err(anyhow!("Invalid permissions!"));
    }

    let command_result = Command::new("chmod")
        .arg(format!("{permissions:o}"))
        .arg(path)
        .status()?;

        if !command_result.success() {
            warn!("Couldn't apply {permissions:o} permissions to '{}' - sensitive data might be accessible to unauthorized third parties!", path.to_string_lossy());
        }
    
    Ok(())
}

pub fn wg_quick_up(config_path: &Path) -> anyhow::Result<()> {
    let command_result = Command::new("sudo")
        .arg("wg-quick")
        .arg("up")
        .arg(config_path)
        .status()?;

    if !command_result.success() {
        return Err(anyhow!("An error occurred when setting up the tunnel interface!"));
    }

    Ok(())
}