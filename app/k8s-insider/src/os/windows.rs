use std::{path::Path, process::Command};

use anyhow::anyhow;
use k8s_insider_core::helpers::escape_quotes_powershell;
use log::debug;

pub fn confine_file_to_owner(path: &Path) -> anyhow::Result<()> {
    // windows_permissions::wrappers::
    Ok(())
}

pub fn install_tunnel_service(config_path: &Path) -> anyhow::Result<()> {
    debug!(
        "Installing tunnel service for '{}'...",
        config_path.to_string_lossy()
    );

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
        .file_stem()
        .ok_or(anyhow!("Invalid config path!"))?;

    debug!(
        "Attempting to remove {} tunnel...",
        config_name.to_string_lossy()
    );

    let command_result = Command::new("wireguard")
        .arg("/uninstalltunnelservice")
        .arg(config_name)
        .status()?;

    if !command_result.success() {
        return Err(anyhow!(
            "An error occurred when removing the tunnel service!"
        ));
    }

    debug!("Tunnel {} removed!", config_name.to_string_lossy());

    Ok(())
}

pub fn add_dns_client_nrpt_rule(domain: &str, dns: &str) -> anyhow::Result<()> {
    let script = format!(
        "Add-DnsClientNrptRule -Namespace \".{}\" -NameServers \"{}\"",
        escape_quotes_powershell(domain),
        escape_quotes_powershell(dns)
    );
    let command_result = Command::new("powershell")
        .arg("-Command")
        .arg(script)
        .status()?;

    if !command_result.success() {
        return Err(anyhow!(
            "An error occurred when patching the Name Resolution Policy Table!"
        ));
    }

    Ok(())
}

pub fn remove_dns_client_nrpt_rule(dns: &str, domain: &str) -> anyhow::Result<()> {
    let script = format!(
        "Get-DnsClientNrptRule \
            | Where-Object {{$_.Namespace -eq \".{domain}\" -and $_.NameServers -eq \"{}\"}} \
            | ForEach-Object -Process {{Remove-DnsClientNrptRule -Name $_.Name -Force}}",
        escape_quotes_powershell(dns)
    );

    let command_result = Command::new("powershell")
        .arg("-Command")
        .arg(script)
        .status()?;

    if !command_result.success() {
        return Err(anyhow!(
            "An error occurred when removing NRPT rules!"
        ));
    }

    Ok(())
}
