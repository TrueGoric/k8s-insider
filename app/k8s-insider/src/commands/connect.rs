use anyhow::{anyhow, Context};
use k8s_insider_core::helpers::RequireMetadata;

use log::info;

use crate::{
    cli::{ConnectArgs, CreateTunnelArgs, GlobalArgs},
    commands::create_tunnel::create_tunnel,
    config::{tunnel::TunnelConfig, ConfigContext, InsiderConfig},
    wireguard::connection::{await_tunnel_availability, tunnel_connect},
};

pub async fn connect(
    global_args: GlobalArgs,
    args: ConnectArgs,
    mut context: ConfigContext,
) -> anyhow::Result<()> {
    let (config_tunnel_name, config_tunnel) =
        match try_get_tunnel_config(args.name.as_deref(), context.insider_config())? {
            Some(config) => config,
            None => {
                info!("No tunnels found in the config, creating a tunnel in a default network...");

                create_tunnel(&global_args, &CreateTunnelArgs::default(), &mut context).await?;

                context.insider_config().try_get_default_tunnel()?.unwrap()
            }
        };

    let (peer_config, _, network) = await_tunnel_availability(config_tunnel, &context).await?;
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

    info!("Successfully connected to the network!");

    Ok(())
}

fn try_get_tunnel_config<'a>(
    name: Option<&str>,
    config: &'a InsiderConfig,
) -> anyhow::Result<Option<(&'a String, &'a TunnelConfig)>> {
    match name {
        Some(name) => Ok(Some(config.try_get_tunnel(name).ok_or(anyhow!(
            "Specified tunnel is not present in the configuration!"
        ))?)),
        None => config.try_get_default_tunnel(),
    }
}
