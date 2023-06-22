use std::fmt::Debug;

use anyhow::Context;
use k8s_openapi::{Metadata, NamespaceResourceScope, serde::{Serialize, de::DeserializeOwned}, api::core::v1::Namespace};
use kube::{Client, api::{PatchParams, Patch, ListParams, DeleteParams}, core::{ObjectMeta, PartialObjectMeta}, Resource, Api, config::{KubeConfigOptions, Kubeconfig}, Config};
use log::{info, warn};

use crate::helpers::pretty_type_name;

pub async fn create_local_client(
    config_path: &Option<String>,
    context_name: &Option<String>,
) -> anyhow::Result<Client> {
    let config_options = KubeConfigOptions {
        context: context_name.to_owned(),
        ..Default::default()
    };

    let config = match config_path {
        Some(path) => {
            let kubeconfig = Kubeconfig::read_from(path)?;
            Config::from_custom_kubeconfig(kubeconfig, &config_options).await?
        }
        None => Config::from_kubeconfig(&config_options).await?,
    };

    let client = Client::try_from(config)?;

    Ok(client)
}

pub async fn create_namespace_if_not_exists(
    client: &Client,
    patch_params: &PatchParams,
    name: &str,
) -> anyhow::Result<()> {
    let namespace_api: Api<Namespace> = Api::all(client.clone());
    let namespace = Namespace {
        metadata: ObjectMeta {
            name: Some(name.to_owned()),
            ..Default::default()
        },
        ..Default::default()
    };

    info!("Ensuring namespace '{}' is created...", name);
    namespace_api
        .patch(&name, patch_params, &Patch::Apply(namespace))
        .await?;

    Ok(())
}

pub async fn try_remove_namespace(
    client: &Client,
    delete_params: &DeleteParams,
    name: &str,
) -> anyhow::Result<()> {
    let namespace_api: Api<Namespace> = Api::all(client.clone());

    namespace_api.delete(name, delete_params).await?;

    Ok(())
}

pub async fn check_if_resource_exists<T: Clone + DeserializeOwned + Debug>(
    release_params: &ListParams,
    api: &Api<T>,
) -> anyhow::Result<bool> {
    let matching_deployments = api
        .list_metadata(&release_params)
        .await
        .context("Couldn't retrieve resources from the cluster!")?;

    match matching_deployments.items.len() {
        0 => Ok(false),
        1 => Ok(true),
        _ => {
            warn!("There are multiple resources matching the release! This could cause unintented behavior!");
            Ok(true)
        }
    }
}

pub async fn create_resource<T>(
    client: &Client,
    resource: &T,
    patch_params: &PatchParams,
) -> anyhow::Result<()>
where
    T: Metadata<Ty = ObjectMeta>
        + Resource<Scope = NamespaceResourceScope, DynamicType = ()>
        + Serialize
        + Clone
        + DeserializeOwned
        + Debug,
{
    info!("Creating the resource on the cluster...");

    let namespace = resource.metadata().namespace.as_ref().unwrap();
    let resource_api: Api<T> = Api::namespaced(client.clone(), namespace);
    let resource_name = resource.metadata().name.as_ref().unwrap();
    resource_api
        .patch(resource_name, patch_params, &Patch::Apply(resource))
        .await
        .context("Unable to create the {}!")?;

    Ok(())
}

pub async fn remove_resources<T: Clone + DeserializeOwned + Debug>(
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
