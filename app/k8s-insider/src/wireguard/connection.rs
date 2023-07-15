use anyhow::{anyhow, Context};
use k8s_insider_core::{
    kubernetes::operations::try_get_resource,
    resources::crd::v1alpha1::{network::Network, tunnel::Tunnel},
    wireguard::keys::WgKey,
};
use kube::Client;

use crate::config::InsiderConfig;

use super::WireguardPeerConfig;

pub async fn get_peer_config(
    name: Option<&str>,
    namespace: &str,
    client: &Client,
    config: InsiderConfig,
) -> anyhow::Result<WireguardPeerConfig> {
    let config_tunnel = match name {
        Some(name) => config.try_get_tunnel(name),
        None => Some(config.try_get_default_tunnel()?),
    };
    let peer_private_key = match config_tunnel {
        Some(config_tunnel) => WgKey::from_base64(&config_tunnel.private_key)
            .context("Invalid private key specified in the configuration!")?,
        None => {
            eprint!("Enter the private key: ");
            WgKey::from_base64_stdin().context("Invalid private key from stdin!")?
        }
    };
    let tunnel_name = config_tunnel
        .map(|t| t.name.as_str())
        .or(name)
        .ok_or(anyhow!(
            "No tunnels found in the config file, you must specify a name!"
        ))?;
    let namespace = config_tunnel
        .map(|c| c.namespace.as_str())
        .unwrap_or(namespace);

    let tunnel = try_get_resource::<Tunnel>(client, tunnel_name, namespace)
        .await?
        .context("Couldn't find the tunnel on the cluster!")?;

    let network = try_get_resource::<Network>(client, &tunnel.spec.network, namespace)
        .await?
        .context("Couldn't find the network on the cluster!")?;

    WireguardPeerConfig::from_crd(peer_private_key, network, tunnel)
        .context("Couldn't create the WireGuard interface configuration!")
}
