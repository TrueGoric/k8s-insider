use std::fmt::Debug;

use anyhow::{anyhow, Context};
use k8s_openapi::api::{
    apps::v1::Deployment,
    core::v1::{ConfigMap, Service},
};
use kube::{api::DeleteParams, core::PartialObjectMeta, Api, Client};
use log::{info, warn};
use serde::de::DeserializeOwned;

use crate::{
    cli::{GlobalArgs, UninstallAllArgs, UninstallArgs},
    helpers::pretty_type_name,
    resources::{
        labels::{get_common_listparams, get_release_listparams},
        namespace::try_remove_namespace,
    },
};

pub async fn uninstall(
    global_args: &GlobalArgs,
    args: &UninstallArgs,
    client: &Client,
) -> anyhow::Result<()> {
    if let Some(release_name) = &args.release_name {
        info!("Uninstalling release '{}'...", release_name);
    } else {
        info!("Uninstalling default release ...");
    }

    let releases_params = match &args.release_name {
        Some(name) => get_release_listparams(name),
        None => get_common_listparams(),
    };

    let deployment_api: Api<Deployment> = Api::namespaced(client.clone(), &global_args.namespace);
    let service_api: Api<Service> = Api::namespaced(client.clone(), &global_args.namespace);

    let deployments = deployment_api
        .list_metadata(&releases_params)
        .await
        .context("Couldn't retrieve release deployments from the cluster!")?;
    let services = service_api
        .list_metadata(&releases_params)
        .await
        .context("Couldn't retrieve release services from the cluster!")?;

    if args.release_name.is_none() && deployments.items.len() > 1 {
        return Err(anyhow!("Multiple releases detected on the cluster!"));
    }

    let delete_params = DeleteParams {
        dry_run: args.dry_run,
        ..Default::default()
    };

    remove_resources(&deployments.items, &delete_params, &deployment_api).await?;
    remove_resources(&services.items, &delete_params, &service_api).await?;

    Ok(())
}

pub async fn uninstall_all(
    global_args: &GlobalArgs,
    args: &UninstallAllArgs,
    client: &Client,
) -> anyhow::Result<()> {
    info!("Uninstalling k8s-insider from the cluster...");

    let releases_params = get_common_listparams();
    let deployment_api: Api<Deployment> = Api::namespaced(client.clone(), &global_args.namespace);
    let configmap_api: Api<ConfigMap> = Api::namespaced(client.clone(), &global_args.namespace);
    let service_api: Api<Service> = Api::namespaced(client.clone(), &global_args.namespace);

    let deployments = deployment_api
        .list_metadata(&releases_params)
        .await
        .context("Couldn't retrieve release deployments from the cluster!")?;
    let configmaps = configmap_api
        .list_metadata(&releases_params)
        .await
        .context("Couldn't retrieve configmaps from the cluster!")?;
    let services = service_api
        .list_metadata(&releases_params)
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
        try_remove_namespace(&client, &delete_params, &global_args.namespace)
            .await
            .context("Couldn't remove the namespace!")?;
    }

    Ok(())
}

async fn remove_resources<T: Clone + DeserializeOwned + Debug>(
    resources: &Vec<PartialObjectMeta<T>>,
    delete_params: &DeleteParams,
    api: &Api<T>,
) -> anyhow::Result<()> {
    let resource_name = pretty_type_name::<T>();
    for service in resources {
        if let Some(name) = &service.metadata.name {
            info!(
                "Removing '{name}' release {} from the cluster...",
                resource_name
            );
            api.delete(&name, &delete_params).await.context(format!(
                "Couldn't delete a release {} from the cluster!",
                resource_name
            ))?;
        } else {
            warn!("Cluster returned a nameless {}!", resource_name); // this shouldn't happen
        }
    }

    Ok(())
}
