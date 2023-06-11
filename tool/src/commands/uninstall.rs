use anyhow::{anyhow, Context};
use k8s_openapi::api::{apps::v1::Deployment, core::v1::ConfigMap};
use kube::{api::DeleteParams, core::PartialObjectMeta, Api, Client};
use log::{info, warn};

use crate::{
    cli::{GlobalArgs, UninstallAllArgs, UninstallArgs},
    resources::{get_common_listparams, get_release_listparams, namespace::try_remove_namespace},
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
    let deployments = deployment_api
        .list_metadata(&releases_params)
        .await
        .context("Couldn't retrieve releases from the cluster!")?;

    if args.release_name.is_none() && deployments.items.len() > 1 {
        return Err(anyhow!("Multiple releases detected on the cluster!"));
    }

    let delete_params = DeleteParams {
        dry_run: args.dry_run,
        ..Default::default()
    };

    remove_deployments(&deployments.items, &delete_params, &deployment_api).await?;

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

    let deployments = deployment_api
        .list_metadata(&releases_params)
        .await
        .context("Couldn't retrieve release deployments from the cluster!")?;
    let configmaps = configmap_api
        .list_metadata(&releases_params)
        .await
        .context("Couldn't retrieve configmaps from the cluster!")?;

    let delete_params = DeleteParams {
        dry_run: args.dry_run,
        ..Default::default()
    };

    remove_deployments(&deployments.items, &delete_params, &deployment_api).await?;
    remove_configmaps(&configmaps.items, &delete_params, &configmap_api).await?;

    if args.delete_namespace {
        info!("Removing namespace '{}'...", global_args.namespace);
        try_remove_namespace(&client, &delete_params, &global_args.namespace)
            .await
            .context("Couldn't remove the namespace!")?;
    }

    Ok(())
}

async fn remove_deployments(
    deployments: &Vec<PartialObjectMeta<Deployment>>,
    delete_params: &DeleteParams,
    deployment_api: &Api<Deployment>,
) -> anyhow::Result<()> {
    for deployment in deployments {
        if let Some(name) = &deployment.metadata.name {
            info!("Removing '{name}' release from the cluster...");
            deployment_api
                .delete(&name, &delete_params)
                .await
                .context("Couldn't delete a release deployment from the cluster!")?;
        } else {
            warn!("Cluster returned a nameless deployment!"); // this shouldn't happen
        }
    }

    Ok(())
}

async fn remove_configmaps(
    configmaps: &Vec<PartialObjectMeta<ConfigMap>>,
    delete_params: &DeleteParams,
    configmap_api: &Api<ConfigMap>,
) -> anyhow::Result<()> {
    for configmap in configmaps {
        if let Some(name) = &configmap.metadata.name {
            info!("Removing '{name}' configmap from the cluster...");
            configmap_api
                .delete(&name, &delete_params)
                .await
                .context("Couldn't delete a configmap from the cluster!")?;
        } else {
            warn!("Cluster returned a nameless configmap!"); // this shouldn't happen
        }
    }

    Ok(())
}
