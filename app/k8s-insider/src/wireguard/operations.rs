use std::path::Path;

use anyhow::Context;

#[cfg(target_os = "linux")]
pub fn tunnel_connect(config_path: &Path) -> anyhow::Result<()> {
    use crate::wireguard::linux::{chmod, wg_quick_up};

    chmod(config_path, 0o600)?;
    wg_quick_up(config_path).context("Couldn't create the WireGuard interface!")?;

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn tunnel_disconnect(config_path: &Path) -> anyhow::Result<()> {
    use crate::wireguard::linux::wg_quick_down;

    wg_quick_down(config_path).context("Couldn't remove the WireGuard interface!")?;

    Ok(())
}
