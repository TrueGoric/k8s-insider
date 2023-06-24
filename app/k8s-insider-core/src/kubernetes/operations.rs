use std::fmt::Debug;

use anyhow::{anyhow, Context};
use k8s_openapi::{
    api::core::v1::Namespace,
    apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition,
    serde::{de::DeserializeOwned, Serialize},
    ClusterResourceScope, Metadata, NamespaceResourceScope,
};
use kube::{
    api::{DeleteParams, ListParams, Patch, PatchParams},
    config::{KubeConfigOptions, Kubeconfig},
    core::{ObjectMeta, PartialObjectMeta},
    Api, Client, Config, Resource,
};
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
        .patch(name, patch_params, &Patch::Apply(namespace))
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
        .list_metadata(release_params)
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
    let resource_name = resource.metadata().name.as_ref().unwrap();

    info!(
        "Creating '{resource_name}' {} resource on the cluster...",
        pretty_type_name::<T>()
    );

    let namespace = resource.metadata().namespace.as_ref().unwrap();
    let resource_api: Api<T> = Api::namespaced(client.clone(), namespace);
    resource_api
        .patch(resource_name, patch_params, &Patch::Apply(resource))
        .await
        .context(format!(
            "Unable to create '{resource_name}' {} resource!",
            pretty_type_name::<T>()
        ))?;

    Ok(())
}

pub async fn create_cluster_resource<T>(
    client: &Client,
    resource: &T,
    patch_params: &PatchParams,
) -> anyhow::Result<()>
where
    T: Metadata<Ty = ObjectMeta>
        + Resource<Scope = ClusterResourceScope, DynamicType = ()>
        + Serialize
        + Clone
        + DeserializeOwned
        + Debug,
{
    let resource_name = resource.metadata().name.as_ref().unwrap();

    info!(
        "Creating '{resource_name}' {} resource on the cluster...",
        pretty_type_name::<T>()
    );

    let resource_api: Api<T> = Api::all(client.clone());
    resource_api
        .patch(resource_name, patch_params, &Patch::Apply(resource))
        .await
        .context(format!(
            "Unable to create '{resource_name}' {} resource!",
            pretty_type_name::<T>()
        ))?;

    Ok(())
}

pub async fn create_crd(
    client: &Client,
    crd: &CustomResourceDefinition,
    patch_params: &PatchParams,
) -> anyhow::Result<()> {
    let crd_name = crd
        .metadata
        .name
        .as_ref()
        .ok_or_else(|| anyhow!("CRD is missing a name!"))?;
    let crd_apiversions = crd
        .spec
        .versions
        .iter()
        .map(|version| version.name.as_str())
        .collect::<Vec<&str>>()
        .join(", ");

    info!("Creating {crd_name} ({crd_apiversions}) CRD...");

    let crd_api: Api<CustomResourceDefinition> = Api::all(client.clone());
    crd_api
        .patch(crd_name, patch_params, &Patch::Apply(crd))
        .await
        .context(format!(
            "Unable to create {crd_name} ({crd_apiversions}) CRD!"
        ))?;

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
            api.delete(name, delete_params).await.context(format!(
                "Couldn't delete a release {} from the cluster!",
                resource_name
            ))?;
        } else {
            warn!("Cluster returned a nameless {}!", resource_name); // this shouldn't happen
        }
    }

    Ok(())
}
