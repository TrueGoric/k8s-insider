use anyhow::anyhow;

use crate::{cli::DisconnectArgs, context::ConfigContext};

pub async fn disconnect(args: DisconnectArgs, mut context: ConfigContext) -> anyhow::Result<()> {
    let config_network = match args.network {
        Some(network) => Some(
            context
                .insider_config
                .try_get_network(&network)
                .ok_or(anyhow!("Couldn't find '{network}' in the config!"))?
                .1,
        ),
        None => None,
    };

    match config_network {
        Some(config_network) => context.connections.remove_connection(&config_network.id)?,
        None => context.connections.remove_single_connection()?,
    }

    Ok(())
}
