use std::path::Path;

use anyhow::Context;

#[cfg(target_os = "linux")]
pub fn tunnel_connect(config_path: &Path) -> anyhow::Result<()> {
    use crate::os::linux::{chmod, wg_quick_up};

    chmod(config_path, 0o600)?;
    wg_quick_up(config_path).context("Couldn't create the WireGuard interface!")?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn tunnel_connect(config_path: &Path) -> anyhow::Result<()> {
    use crate::os::windows::{confine_file_to_owner, install_tunnel_service};

    confine_file_to_owner(config_path)?;
    install_tunnel_service(config_path).context(
        "Couldn't create WireGuard tunnel service! Administrator privileges might be required.",
    )?;

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn tunnel_disconnect(config_path: &Path) -> anyhow::Result<()> {
    use crate::os::linux::wg_quick_down;

    wg_quick_down(config_path).context("Couldn't remove the WireGuard interface!")?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn tunnel_disconnect(config_path: &Path) -> anyhow::Result<()> {
    use crate::os::windows::uninstall_tunnel_service;

    uninstall_tunnel_service(config_path).context(
        "Couldn't remove the WireGuard tunnel service! Administrator privileges might be required.",
    )?;

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn patch_dns_linux(ifname: &str, domain: &str) -> anyhow::Result<()> {
    use crate::os::linux::resolvectl_domain;

    resolvectl_domain(ifname, domain)?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn patch_dns_windows(dns: &str, domain: &str) -> anyhow::Result<()> {
    use crate::os::windows::add_dns_client_nrpt_rule;

    add_dns_client_nrpt_rule(domain, dns)?;

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn unpatch_dns(dns: &str) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn unpatch_dns(dns: &str, domain: &str) -> anyhow::Result<()> {
    use crate::os::windows::remove_dns_client_nrpt_rule;

    remove_dns_client_nrpt_rule(dns, domain)?;

    Ok(())
}
