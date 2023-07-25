use log::{info, warn};

use crate::{cli::ConfigRemoveNetworkArgs, context::ConfigContext};

pub fn config_remove_network(
    args: ConfigRemoveNetworkArgs,
    mut context: ConfigContext,
) -> anyhow::Result<()> {
    info!("Removing '{}' network from config...", args.name);

    let insider_config = context.insider_config_mut();

    if insider_config.try_remove_network(&args.name).is_ok() {
        insider_config.save()?;
        info!("Network successfully removed!");
    } else {
        warn!("There's no network named '{}' in the config!", {
            args.name
        });
    }

    Ok(())
}
