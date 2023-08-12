use anyhow::anyhow;
use k8s_insider_core::helpers::RequireMetadata;

use log::info;

use crate::{
    cli::{ConnectArgs, CreateTunnelArgs, GlobalArgs},
    commands::create_tunnel::create_tunnel,
    config::{network::NetworkConfig, tunnel::TunnelConfig},
    context::ConfigContext,
    wireguard::helpers::await_tunnel_availability,
};

pub async fn connect(
    global_args: GlobalArgs,
    args: ConnectArgs,
    mut context: ConfigContext,
) -> anyhow::Result<()> {
    let (config_network_name, mut config_network) = context
        .insider_config
        .get_network_or_default(args.network.as_deref())?;
    let mut config_tunnel_opt = try_get_tunnel_config(args.name.as_deref(), config_network)?;

    if config_tunnel_opt.is_none() {
        info!("No tunnels found locally in the network, creating one...");

        let create_args = CreateTunnelArgs {
            network: Some(config_network_name.to_owned()),
            name: None,
            static_ip: None,
        };
        create_tunnel(&global_args, &create_args, &mut context).await?;

        (_, config_network) = context
            .insider_config
            .get_network_or_default(args.network.as_deref())?;
        config_tunnel_opt = try_get_tunnel_config(args.name.as_deref(), config_network)?;
    }
    let config_network = config_network;
    let config_tunnel = config_tunnel_opt.unwrap().1;
    let (peer_meta, peer_config, _, network) =
        await_tunnel_availability(config_network, config_tunnel, &context).await?;
    let network_name = network.require_name_or(anyhow!("Network CRD doesn't have a name!"))?;

    info!(
        "Connecting to '{}' network in '{}' namespace...",
        network_name, global_args.namespace
    );

    context
        .connections
        .create_connection(peer_meta, peer_config)?;

    info!("Tunnel link created...");

    // TODO: check connectivity

    info!("Successfully connected to the network!");

    Ok(())
}

fn try_get_tunnel_config<'a>(
    name: Option<&str>,
    config_network: &'a NetworkConfig,
) -> anyhow::Result<Option<(&'a String, &'a TunnelConfig)>> {
    match name {
        Some(name) => Ok(Some(config_network.try_get_tunnel(name).ok_or(anyhow!(
            "Specified tunnel is not present in the configuration!"
        ))?)),
        None => config_network.try_get_default_tunnel(),
    }
}
