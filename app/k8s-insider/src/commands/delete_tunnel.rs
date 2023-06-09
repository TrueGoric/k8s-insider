use anyhow::{anyhow, Context};
use k8s_insider_core::{
    helpers::AndIf, kubernetes::operations::try_remove_resource,
    resources::crd::v1alpha1::tunnel::Tunnel,
};
use kube::{api::DeleteParams, Client};
use log::info;

use crate::{
    cli::{DeleteTunnelArgs, GlobalArgs},
    config::InsiderConfig,
};

pub async fn delete_tunnel(
    _global_args: GlobalArgs,
    args: DeleteTunnelArgs,
    client: Client,
    mut config: InsiderConfig,
) -> anyhow::Result<()> {
    let tunnel = config.tunnels.remove(&args.name).ok_or(anyhow!(
        "'{}' tunnel is not defined in the config!",
        args.name
    ))?;

    info!("Removing '{}' tunnel...", args.name);

    let delete_params = DeleteParams::background().and_if(args.dry_run, DeleteParams::dry_run);
    let was_removed =
        try_remove_resource::<Tunnel>(&client, &tunnel.name, &tunnel.namespace, &delete_params)
            .await?;

    config
        .save()
        .context("Couldn't save the configuration file!")?;

    if was_removed {
        info!("Tunnel successfully deleted!");
    } else {
        info!("Couldn't find '{}' network on the cluster!", args.name);
    }

    Ok(())
}
