use anyhow::Context;

use kube::Client;

use crate::{
    cli::{GetConfArgs, GlobalArgs},
    config::InsiderConfig,
    wireguard::connection::get_peer_config,
};

pub async fn get_configuration(
    global_args: GlobalArgs,
    args: GetConfArgs,
    client: Client,
    config: InsiderConfig,
) -> anyhow::Result<()> {
    let peer_config = get_peer_config(
        args.name.as_deref(),
        &global_args.namespace,
        &client,
        config,
    )
    .await?;

    if let Some(output) = args.output {
        std::fs::write(&output, peer_config.generate_configuration_file())
            .context(format!("Couldn't write the output to {output}"))?;
    } else {
        print!("{}", peer_config.generate_configuration_file());
    }

    Ok(())
}
