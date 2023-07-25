use anyhow::{anyhow, Context};
use k8s_insider_core::helpers::RequireMetadata;

use log::info;

use crate::{
    cli::{ConnectArgs, CreateTunnelArgs, GlobalArgs},
    commands::create_tunnel::create_tunnel,
    config::{network::NetworkConfig, tunnel::TunnelConfig, InsiderConfig},
    context::ConfigContext,
    wireguard::{connection::tunnel_connect, helpers::await_tunnel_availability},
};

pub async fn connect(
    global_args: GlobalArgs,
    args: ConnectArgs,
    mut context: ConfigContext,
) -> anyhow::Result<()> {
    let (config_network, config_tunnel_name, config_tunnel) =
        match try_get_configs(&args, &context)? {
            Some(config_tuple) => config_tuple,
            None => {
                info!("No tunnels found locally in the network, creating one...");

                create_tunnel(&global_args, &CreateTunnelArgs::default(), &mut context).await?;

                try_get_configs(&args, &context)?.unwrap()
            }
        };

    let (peer_config, _, network) =
        await_tunnel_availability(config_network, config_tunnel, &context).await?;
    let network_name = network.require_name_or(anyhow!("Network CRD doesn't have a name!"))?;

    info!(
        "Connecting to '{}' network in '{}' namespace...",
        network_name, global_args.namespace
    );

    let peer_config_name = format!("{}.conf", config_tunnel_name);
    let peer_config_path = context.get_path_base().join(peer_config_name);

    peer_config
        .write_configuration(&peer_config_path)
        .context(format!(
            "Couldn't write the configuration file to '{}'!",
            peer_config_path.to_string_lossy()
        ))?;

    info!(
        "WireGuard config written to '{}'...",
        peer_config_path.to_string_lossy()
    );

    tunnel_connect(&peer_config_path)?;
    info!("Tunnel link created...");

    // TODO: check connectivity

    info!("Successfully connected to the network!");

    Ok(())
}

fn try_get_configs<'a>(
    args: &ConnectArgs,
    context: &'a ConfigContext,
) -> anyhow::Result<Option<(&'a NetworkConfig, &'a String, &'a TunnelConfig)>> {
    let (_, config_network) =
        get_network_config(args.network.as_deref(), context.insider_config())?;
    let config_tunnel = match try_get_tunnel_config(args.name.as_deref(), config_network)? {
        Some(config) => Some(config),
        None => config_network.try_get_default_tunnel()?,
    };

    if let Some((config_tunnel_name, config_tunnel)) = config_tunnel {
        Ok(Some((config_network, config_tunnel_name, config_tunnel)))
    } else {
        Ok(None)
    }
}

fn get_network_config<'a>(
    name: Option<&str>,
    config: &'a InsiderConfig,
) -> anyhow::Result<(&'a String, &'a NetworkConfig)> {
    match name {
        Some(name) => Ok(config.try_get_network(name).ok_or(anyhow!(
            "Specified network is not present in the configuration!"
        ))?),
        None => Ok(config
            .try_get_default_network()?
            .ok_or(anyhow!("No networks defined in the config!"))?),
    }
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
