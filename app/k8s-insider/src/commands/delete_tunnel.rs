use anyhow::{anyhow, Context};
use k8s_insider_core::{
    helpers::AndIf, kubernetes::operations::try_remove_resource,
    resources::crd::v1alpha1::tunnel::Tunnel,
};
use kube::api::DeleteParams;
use log::info;

use crate::{
    cli::{DeleteTunnelArgs, GlobalArgs},
    config::ConfigContext,
};

pub async fn delete_tunnel(
    _global_args: GlobalArgs,
    args: DeleteTunnelArgs,
    mut context: ConfigContext,
) -> anyhow::Result<()> {
    let client = context.create_client_with_default_context().await?;

    let network = context
        .insider_config_mut()
        .try_get_network_mut(&args.network)
        .ok_or(anyhow!(
            "Couldn't find '{}' network in the config!",
            args.network
        ))?;
    let tunnel = network.try_remove_tunnel(&args.tunnel)?;

    info!("Removing '{}' tunnel...", args.tunnel);

    let delete_params = DeleteParams::background().and_if(args.dry_run, DeleteParams::dry_run);
    let was_removed =
        try_remove_resource::<Tunnel>(&client, &tunnel.name, &network.id.namespace, &delete_params)
            .await?;

    context
        .insider_config()
        .save()
        .context("Couldn't save the configuration file!")?;

    if was_removed {
        info!("Tunnel successfully deleted!");
    } else {
        info!("Couldn't find '{}' network on the cluster!", args.tunnel);
    }

    Ok(())
}
