use anyhow::Context;
use k8s_insider_core::{
    kubernetes::operations::{remove_resources, try_remove_namespace},
    resources::labels::get_router_listparams,
};
use k8s_openapi::api::{
    apps::v1::Deployment,
    core::v1::{ConfigMap, Service},
};
use kube::{api::DeleteParams, Api, Client};
use log::info;

use crate::cli::{GlobalArgs, UninstallArgs};

pub async fn uninstall(
    global_args: &GlobalArgs,
    args: &UninstallArgs,
    client: &Client,
) -> anyhow::Result<()> {
    info!("Uninstalling k8s-insider from '{}' namespace...", global_args.namespace);

    let tunnel_params = get_router_listparams();
    let deployment_api: Api<Deployment> = Api::namespaced(client.clone(), &global_args.namespace);
    let configmap_api: Api<ConfigMap> = Api::namespaced(client.clone(), &global_args.namespace);
    let service_api: Api<Service> = Api::namespaced(client.clone(), &global_args.namespace);

    let deployments = deployment_api
        .list_metadata(&tunnel_params)
        .await
        .context("Couldn't retrieve release deployments from the cluster!")?;
    let configmaps = configmap_api
        .list_metadata(&tunnel_params)
        .await
        .context("Couldn't retrieve configmaps from the cluster!")?;
    let services = service_api
        .list_metadata(&tunnel_params)
        .await
        .context("Couldn't retrieve release services from the cluster!")?;

    let delete_params = DeleteParams {
        dry_run: args.dry_run,
        ..Default::default()
    };

    remove_resources(&deployments.items, &delete_params, &deployment_api).await?;
    remove_resources(&configmaps.items, &delete_params, &configmap_api).await?;
    remove_resources(&services.items, &delete_params, &service_api).await?;

    if args.delete_namespace {
        info!("Removing namespace '{}'...", global_args.namespace);
        try_remove_namespace(client, &delete_params, &global_args.namespace)
            .await
            .context("Couldn't remove the namespace!")?;
    }

    Ok(())
}
