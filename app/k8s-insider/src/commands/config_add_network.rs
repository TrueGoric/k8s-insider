use log::info;

use crate::{
    cli::{ConfigAddNetworkArgs, GlobalArgs},
    config::network::NetworkIdentifier,
    context::ConfigContext,
};

pub fn config_add_network(
    global_args: GlobalArgs,
    args: ConfigAddNetworkArgs,
    mut context: ConfigContext,
) -> anyhow::Result<()> {
    info!("Adding '{}' network to the config...", args.name);

    let kube_context = global_args
        .kube_context
        .unwrap_or_else(|| context.kube_context_name().to_owned());

    let local_name = args.local_name.unwrap_or_else(|| args.name.to_owned());

    let network = NetworkIdentifier::new(args.name, global_args.namespace, kube_context).into();
    let config = context.insider_config_mut();

    config.try_add_network(local_name, network)?;
    config.save()?;

    info!("Network successfully added!");

    Ok(())
}
