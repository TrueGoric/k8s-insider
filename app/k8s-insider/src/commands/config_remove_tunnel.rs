use log::{info, warn};

use crate::{cli::ConfigRemoveTunnelArgs, config::ConfigContext};

pub fn config_remove_tunnel(
    args: ConfigRemoveTunnelArgs,
    mut context: ConfigContext,
) -> anyhow::Result<()> {
    info!("Removing '{}' tunnel from config...", args.name);

    let insider_config = context.insider_config_mut();
    if insider_config.tunnels.remove(&args.name).is_some() {
        insider_config.save()?;
        info!("Tunnel successfully removed!");
    } else {
        warn!("There's no tunnel named '{}' defined in the config!", {
            args.name
        });
    }

    Ok(())
}
