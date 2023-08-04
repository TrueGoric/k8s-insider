use std::{path::Path, process::Command};

use anyhow::anyhow;

pub fn confine_file_to_owner(path: &Path) -> anyhow::Result<()> {
    // windows_permissions::wrappers::
    Ok(())
}

pub fn install_tunnel_service(config_path: &Path) -> anyhow::Result<()> {
    let command_result = Command::new("wireguard")
        .arg("/installtunnelservice")
        .arg(config_path)
        .status()?;

    if !command_result.success() {
        return Err(anyhow!(
            "An error occurred when setting up the tunnel service!"
        ));
    }

    Ok(())
}

pub fn uninstall_tunnel_service(config_path: &Path) -> anyhow::Result<()> {
    let config_name = config_path
        .file_name()
        .ok_or(anyhow!("Invalid config path!"))?;

    let command_result = Command::new("wireguard")
        .arg("/uninstalltunnelservice")
        .arg(config_name)
        .status()?;

    if !command_result.success() {
        return Err(anyhow!(
            "An error occurred when removing the tunnel service!"
        ));
    }

    Ok(())
}

pub fn add_dns_client_nrpt_rule(domain: &str, dns: &str) {
    // Add-DnsClientNrptRule -Namespace "pqr.com" -NameServers "10.0.0.1"
}