use anyhow::anyhow;
use log::{info, warn};

use crate::{cli::ConfigRemoveTunnelArgs, context::ConfigContext};

pub fn config_remove_tunnel(
    args: ConfigRemoveTunnelArgs,
    mut context: ConfigContext,
) -> anyhow::Result<()> {
    info!("Removing '{}' tunnel from config...", args.name);

    let insider_config = context.insider_config_mut();
    let config_network = insider_config
        .try_get_network_mut(&args.network)
        .ok_or(anyhow!(
            "'{}' network is not present in the config!",
            args.network
        ))?;

    if config_network.try_remove_tunnel(&args.name).is_ok() {
        insider_config.save()?;
        info!("Tunnel successfully removed!");
    } else {
        warn!("There's no tunnel named '{}' defined in the config!", {
            args.name
        });
    }

    Ok(())
}
