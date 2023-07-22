use crate::{
    cli::{ConfigAddNetworkArgs, GlobalArgs},
    config::{network::NetworkConfig, ConfigContext},
};

pub fn config_add_network(
    global_args: GlobalArgs,
    args: ConfigAddNetworkArgs,
    mut context: ConfigContext,
) -> anyhow::Result<()> {
    let kube_context = global_args
        .kube_context
        .unwrap_or_else(|| context.kube_context_name().to_owned());

    let network = NetworkConfig::new(global_args.namespace, kube_context);
    let config = context.insider_config_mut();

    config.try_add_network(args.name, network)?;
    config.save()?;

    Ok(())
}
