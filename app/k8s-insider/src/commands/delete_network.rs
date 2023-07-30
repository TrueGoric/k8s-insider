use anyhow::Context;
use k8s_insider_core::{
    helpers::AndIf, kubernetes::operations::try_remove_resource,
    resources::crd::v1alpha1::network::Network,
};
use kube::api::DeleteParams;
use log::info;

use crate::{
    cli::{DeleteNetworkArgs, GlobalArgs},
    context::ConfigContext,
};

pub async fn delete_network(
    global_args: GlobalArgs,
    args: DeleteNetworkArgs,
    mut context: ConfigContext,
) -> anyhow::Result<()> {
    let config_network = context
        .insider_config_mut()
        .try_remove_network(&args.name)?;

    let client = context.create_client(&config_network.id.context).await?;

    info!(
        "Removing '{}' network in '{}' namespace...",
        args.name, global_args.namespace
    );

    let delete_params = DeleteParams::background().and_if(args.dry_run, DeleteParams::dry_run);
    let was_removed =
        try_remove_resource::<Network>(&client, &args.name, &global_args.namespace, &delete_params)
            .await?;

    if was_removed {
        context
            .insider_config()
            .save()
            .context("Couldn't save the configuration file!")?;

        info!("Network successfully deleted!");
    } else {
        info!("Couldn't find '{}' network on the cluster!", args.name);
    }

    Ok(())
}
