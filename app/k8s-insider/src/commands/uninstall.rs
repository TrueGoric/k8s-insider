use anyhow::Context;
use k8s_insider_core::{
    kubernetes::operations::{
        remove_matching_resources, try_remove_cluster_resource, try_remove_namespace,
    },
    resources::{crd::v1alpha1::remove_v1alpha1_crds, labels::get_controller_listparams},
    CONTROLLER_CLUSTERROLE_NAME, NETWORK_MANAGER_CLUSTERROLE_NAME, ROUTER_CLUSTERROLE_NAME,
};
use k8s_openapi::api::{
    apps::v1::Deployment,
    core::v1::{ConfigMap, ServiceAccount},
    rbac::v1::{ClusterRole, ClusterRoleBinding},
};
use kube::api::DeleteParams;
use log::info;

use crate::{
    cli::{GlobalArgs, UninstallArgs},
    config::ConfigContext,
};

pub async fn uninstall(
    global_args: GlobalArgs,
    args: UninstallArgs,
    context: ConfigContext,
) -> anyhow::Result<()> {
    let client = context.create_client_with_default_context().await?;

    info!(
        "Uninstalling k8s-insider from '{}' namespace...",
        global_args.namespace
    );

    let list_params = get_controller_listparams();
    let mut del_params = DeleteParams::default();

    if args.dry_run {
        del_params = del_params.dry_run();
    };

    remove_matching_resources::<Deployment>(&client, &list_params, &del_params).await?;
    remove_matching_resources::<ConfigMap>(&client, &list_params, &del_params).await?;
    remove_matching_resources::<ServiceAccount>(&client, &list_params, &del_params).await?;

    try_remove_cluster_resource::<ClusterRole>(&client, CONTROLLER_CLUSTERROLE_NAME, &del_params)
        .await?;
    try_remove_cluster_resource::<ClusterRole>(
        &client,
        NETWORK_MANAGER_CLUSTERROLE_NAME,
        &del_params,
    )
    .await?;
    try_remove_cluster_resource::<ClusterRole>(&client, ROUTER_CLUSTERROLE_NAME, &del_params)
        .await?;
    try_remove_cluster_resource::<ClusterRoleBinding>(
        &client,
        CONTROLLER_CLUSTERROLE_NAME,
        &del_params,
    )
    .await?;

    if !args.leave_crds {
        info!("Removing v1alpha1 CRDs...");
        remove_v1alpha1_crds(&client, args.dry_run).await?;
    }

    if args.delete_namespace {
        info!("Removing namespace '{}'...", global_args.namespace);
        try_remove_namespace(&client, &del_params, &global_args.namespace)
            .await
            .context("Couldn't remove the namespace!")?;
    }

    info!("Successfully removed k8s-insider!");

    Ok(())
}
