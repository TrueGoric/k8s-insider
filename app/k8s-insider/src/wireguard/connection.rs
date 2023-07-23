use std::{path::Path, time::Duration};

use anyhow::{anyhow, Context};
use k8s_insider_core::{
    kubernetes::operations::{await_resource_condition, AwaitError},
    resources::crd::v1alpha1::{network::Network, tunnel::Tunnel},
};

use crate::config::{network::NetworkConfig, tunnel::TunnelConfig, ConfigContext};

use super::WireguardPeerConfig;

pub async fn await_tunnel_availability(
    config_network: &NetworkConfig,
    config_tunnel: &TunnelConfig,
    context: &ConfigContext,
) -> anyhow::Result<(WireguardPeerConfig, Tunnel, Network)> {
    let client = context.create_client(&config_network.id.context).await?;

    let tunnel_condition = |t: Option<&Tunnel>| t.map(|t| t.is_ready() || t.is_error()).unwrap_or(true);
    let tunnel = await_resource_condition::<Tunnel>(
        &client,
        &config_tunnel.name,
        &config_network.id.namespace,
        tunnel_condition,
        Duration::from_secs(5),
    )
    .await;

    let tunnel = match tunnel {
        Ok(tunnel) => tunnel,
        Err(error) => match error {
            AwaitError::Timeout(tunnel) => tunnel,
            _ => return Err(error.into()),
        },
    }
    .context("Couldn't find the tunnel on the cluster!")?;

    if tunnel.is_error() || tunnel.is_closed() {
        return Err(anyhow!(
            "Tunnel is in an invalid state ({:?})!",
            tunnel.status.map(|s| s.state)
        ));
    }

    if !tunnel.is_ready() {
        return Err(anyhow!("Timed out waiting for tunnel to be ready!"));
    }

    let network_condition = |n: Option<&Network>| n.map(|n| n.is_ready() || n.is_error()).unwrap_or(true);
    let network = await_resource_condition::<Network>(
        &client,
        &tunnel.spec.network,
        &config_network.id.namespace,
        network_condition,
        Duration::from_secs(5),
    )
    .await;

    let network = match network {
        Ok(network) => network,
        Err(error) => match error {
            AwaitError::Timeout(network) => network,
            _ => return Err(error.into()),
        },
    }
    .context("Couldn't find the network on the cluster!")?;

    if network.is_error() {
        return Err(anyhow!(
            "Network is in an invalid state ({:?})!",
            network.status.map(|s| s.state)
        ));
    }

    if !network.is_ready() {
        return Err(anyhow!("Timed out waiting for network to be ready!"));
    }

    let peer_config = get_peer_config(config_tunnel, &network, &tunnel)?;

    Ok((peer_config, tunnel, network))
}

pub fn get_peer_config(
    config_tunnel: &TunnelConfig,
    network: &Network,
    tunnel: &Tunnel,
) -> anyhow::Result<WireguardPeerConfig> {
    let peer_private_key = config_tunnel.try_get_wgkey().context(format!(
        "Invalid key specified in the config for tunnel '{}'!",
        config_tunnel.name
    ))?;

    let peer_config = WireguardPeerConfig::from_crd(peer_private_key, network, tunnel)
        .context("Couldn't create the WireGuard interface configuration!")?;

    Ok(peer_config)
}

#[cfg(target_os = "linux")]
pub fn tunnel_connect(config_path: &Path) -> anyhow::Result<()> {
    use crate::wireguard::linux::{chmod, wg_quick_up};

    chmod(config_path, 0o600)?;
    wg_quick_up(config_path).context("Couldn't create the WireGuard interface!")?;

    Ok(())
}
