use std::path::Path;

use anyhow::{anyhow, Context};

use crate::os::linux::{chmod, wg_quick_down, wg_quick_up};

pub fn tunnel_connect(config_path: &Path) -> anyhow::Result<()> {
    if cfg!(target_os = "linux") {
        chmod(config_path, 0o600)?;
        wg_quick_up(config_path).context("Couldn't create the WireGuard interface!")?;
    } else {
        return Err(anyhow!(
            "Can't create a WireGuard tunnel! {} is an unsupported OS for this operation!",
            std::env::consts::OS
        ));
    }

    Ok(())
}

pub fn tunnel_disconnect(config_path: &Path) -> anyhow::Result<()> {
    if cfg!(target_os = "linux") {
        wg_quick_down(config_path).context("Couldn't remove the WireGuard interface!")?;
    } else {
        return Err(anyhow!(
            "Can't remove a WireGuard tunnel! {} is an unsupported OS for this operation!",
            std::env::consts::OS
        ));
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn patch_dns_linux(ifname: &str, domain: &str) -> anyhow::Result<()> {
    use crate::os::linux::resolvectl_domain;

    resolvectl_domain(ifname, domain)?;
    Ok(())
}
