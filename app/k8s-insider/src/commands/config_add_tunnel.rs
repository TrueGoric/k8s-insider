use anyhow::{anyhow, Context};
use k8s_insider_core::wireguard::keys::WgKey;

use crate::{cli::ConfigAddTunnelArgs, config::tunnel::TunnelConfig, context::ConfigContext};

pub fn config_add_tunnel(
    args: ConfigAddTunnelArgs,
    mut context: ConfigContext,
) -> anyhow::Result<()> {
    eprint!("Enter the private key: ");
    let private_key = WgKey::from_base64_stdin()
        .context("Invalid private key from stdin!")?
        .to_base64();
    let config = context.insider_config_mut();
    let config_network = config.try_get_network_mut(&args.network).ok_or(anyhow!(
        "'{}' network is not present in the config!",
        args.network
    ))?;
    let config_tunnel = TunnelConfig {
        name: args.name,
        private_key,
    };

    let local_name = args
        .local_name
        .unwrap_or_else(|| config_network.generate_config_tunnel_name());

    config_network.try_add_tunnel(local_name, config_tunnel)?;
    config.save()?;

    Ok(())
}
