use anyhow::Context;
use k8s_insider_core::{
    kubernetes::operations::try_get_resource,
    resources::crd::v1alpha1::{network::Network, tunnel::Tunnel},
};

use crate::config::{tunnel::TunnelConfig, ConfigContext};

use super::WireguardPeerConfig;

pub async fn get_peer_config(
    config_tunnel: &TunnelConfig,
    context: &ConfigContext,
) -> anyhow::Result<(WireguardPeerConfig, Tunnel, Network)> {
    let peer_private_key = config_tunnel.try_get_wgkey().context(format!(
        "Invalid key specified in the config for tunnel '{}'!",
        config_tunnel.name
    ))?;

    let client = context.create_client(&config_tunnel.context).await?;
    let tunnel = try_get_resource::<Tunnel>(&client, &config_tunnel.name, &config_tunnel.namespace)
        .await?
        .context("Couldn't find the tunnel on the cluster!")?;

    let network =
        try_get_resource::<Network>(&client, &tunnel.spec.network, &config_tunnel.namespace)
            .await?
            .context("Couldn't find the network on the cluster!")?;

    let peer_config = WireguardPeerConfig::from_crd(peer_private_key, &network, &tunnel)
        .context("Couldn't create the WireGuard interface configuration!")?;

    Ok((peer_config, tunnel, network))
}
