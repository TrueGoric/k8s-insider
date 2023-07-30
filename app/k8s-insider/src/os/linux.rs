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
    let command_result = Command::new("wg-quick")
        .arg("up")
        .arg(config_path)
        .status()?;

    if !command_result.success() {
        return Err(anyhow!(
            "An error occurred when setting up the tunnel interface!"
        ));
    }

    Ok(())
}

pub fn wg_quick_down(config_path: &Path) -> anyhow::Result<()> {
    let command_result = Command::new("wg-quick")
        .arg("down")
        .arg(config_path)
        .status()?;

    if !command_result.success() {
        return Err(anyhow!(
            "An error occurred when removing the tunnel interface!"
        ));
    }

    Ok(())
}

pub fn resolvectl_domain(ifname: &str, domain: &str) -> anyhow::Result<()> {
    let command_result = Command::new("resolvectl")
        .arg("domain")
        .arg(ifname)
        .arg(domain)
        .status()?;

    if !command_result.success() {
        return Err(anyhow!(
            "An error occurred when patching DNS with resolvectl!"
        ));
    }

    Ok(())
}
