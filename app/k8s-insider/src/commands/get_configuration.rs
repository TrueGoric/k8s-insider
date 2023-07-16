use std::path::Path;

use anyhow::Context;

use crate::{
    cli::{GetConfArgs, GlobalArgs},
    config::ConfigContext,
    wireguard::connection::get_peer_config,
};

pub async fn get_configuration(
    global_args: GlobalArgs,
    args: GetConfArgs,
    context: ConfigContext,
) -> anyhow::Result<()> {
    let client = context.create_client_with_default_context().await?;
    let (peer_config, _, _) = get_peer_config(
        args.name.as_deref(),
        &global_args.namespace,
        &client,
        context.insider_config(),
    )
    .await?;

    if let Some(output) = args.output {
        peer_config
            .write_configuration(Path::new(&output))
            .context(format!("Couldn't write the output to {output}"))?;
    } else {
        print!("{}", peer_config.generate_configuration_file());
    }

    Ok(())
}
