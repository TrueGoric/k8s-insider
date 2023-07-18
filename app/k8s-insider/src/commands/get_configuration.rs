use std::path::Path;

use anyhow::{anyhow, Context};

use crate::{cli::GetConfArgs, config::ConfigContext, wireguard::connection::get_peer_config};

pub async fn get_configuration(args: GetConfArgs, context: ConfigContext) -> anyhow::Result<()> {
    let config = context.insider_config();
    let config_tunnel = match args.name {
        Some(name) => config.try_get_tunnel(&name).ok_or(anyhow!(
            "Specified tunnel is not present in the configuration!"
        ))?,
        None => config
            .try_get_default_tunnel()?
            .ok_or(anyhow!("No tunnels were written to the config!"))?,
    };

    let (peer_config, _, _) = get_peer_config(config_tunnel, &context).await?;

    if let Some(output) = args.output {
        peer_config
            .write_configuration(Path::new(&output))
            .context(format!("Couldn't write the output to {output}"))?;
    } else {
        print!("{}", peer_config.generate_configuration_file());
    }

    Ok(())
}
