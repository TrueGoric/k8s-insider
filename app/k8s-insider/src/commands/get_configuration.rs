use std::path::Path;

use anyhow::{anyhow, Context};

use crate::{
    cli::GetConfArgs, config::ConfigContext, wireguard::connection::await_tunnel_availability,
};

pub async fn get_configuration(args: GetConfArgs, context: ConfigContext) -> anyhow::Result<()> {
    let config = context.insider_config();
    let (network_name, config_network) = match args.network {
        Some(name) => config.try_get_network(&name).ok_or(anyhow!(
            "Specified network is not present in the configuration!"
        ))?,
        None => config
            .try_get_default_network()?
            .ok_or(anyhow!("No networks are present in the config!"))?,
    };
    let config_tunnel = match args.tunnel {
        Some(name) => {
            config_network
                .try_get_tunnel(&name)
                .ok_or(anyhow!(
                    "Specified tunnel is not present in '{network_name}'network configuration!"
                ))?
                .1
        }
        None => {
            config_network
                .try_get_default_tunnel()?
                .ok_or(anyhow!(
                    "No tunnels specified in '{network_name}' network configuration!"
                ))?
                .1
        }
    };

    let (peer_config, _, _) =
        await_tunnel_availability(config_network, config_tunnel, &context).await?;

    if let Some(output) = args.output {
        peer_config
            .write_configuration(Path::new(&output))
            .context(format!("Couldn't write the output to {output}"))?;
    } else {
        print!("{}", peer_config.generate_configuration_file());
    }

    Ok(())
}
