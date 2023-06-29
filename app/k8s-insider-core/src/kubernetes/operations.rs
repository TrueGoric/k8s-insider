use std::fmt::Debug;

use anyhow::{anyhow, Context};
use futures::Stream;
use k8s_openapi::{
    api::core::v1::Namespace,
    apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition,
    serde::{de::DeserializeOwned, Serialize},
    ClusterResourceScope, NamespaceResourceScope,
};
use kube::{
    api::{DeleteParams, ListParams, Patch, PatchParams},
    config::{KubeConfigOptions, Kubeconfig},
    core::{object::HasStatus, ObjectMeta},
    runtime::watcher::{self, watch_object},
    Api, Client, Config, Resource,
};
use log::{debug, info, warn};

use crate::helpers::pretty_type_name;

use super::FromStatus;

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

pub fn watch_resource<T>(
    client: &Client,
    resource_name: &str,
    namespace: &str,
) -> impl Stream<Item = Result<Option<T>, watcher::Error>>
where
    T: Resource<Scope = NamespaceResourceScope> + Clone + DeserializeOwned + Debug + Send + 'static,
    <T as Resource>::DynamicType: Default,
{
    let api: Api<T> = Api::namespaced(client.clone(), namespace);

    watch_object(api, resource_name)
}

pub async fn try_get_resource<T>(
    client: &Client,
    resource_name: &str,
    namespace: &str,
) -> Result<Option<T>, kube::Error>
where
    T: Resource<Scope = NamespaceResourceScope> + Serialize + Clone + DeserializeOwned + Debug,
    <T as Resource>::DynamicType: Default,
{
    let response = Api::namespaced(client.clone(), namespace)
        .get(resource_name)
        .await;

    match response {
        Ok(resource) => Ok(Some(resource)),
        Err(error) => match &error {
            kube::Error::Api(api_error) => match api_error.code {
                404 => Ok(None),
                _ => Err(error),
            },
            _ => Err(error),
        },
    }
}

pub async fn list_resources<T>(
    client: &Client,
    namespace: &str,
    list_params: &ListParams,
) -> Result<Vec<T>, kube::Error>
where
    T: Resource<Scope = NamespaceResourceScope> + Serialize + Clone + DeserializeOwned + Debug,
    <T as Resource>::DynamicType: Default,
{
    let response = Api::namespaced(client.clone(), namespace)
        .list(list_params)
        .await?;

    Ok(response.items)
}

pub async fn create_namespace_if_not_exists(
    client: &Client,
    patch_params: &PatchParams,
    name: &str,
) -> Result<(), kube::Error> {
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

pub async fn apply_resource<T>(
    client: &Client,
    resource: &T,
    patch_params: &PatchParams,
) -> Result<(), kube::Error>
where
    T: Resource<Scope = NamespaceResourceScope> + Serialize + Clone + DeserializeOwned + Debug,
    <T as Resource>::DynamicType: Default,
{
    let resource_type_name = pretty_type_name::<T>();
    let resource_name = resource.meta().name.as_ref().unwrap();

    info!("Applying '{resource_name}' {resource_type_name} resource on the cluster...",);

    debug!(
        "{}",
        serde_json::to_string_pretty(&resource)
            .unwrap_or(format!("Couldn't serialize '{resource_name}'"))
    );

    let namespace = resource.meta().namespace.as_ref().unwrap();
    let resource_api: Api<T> = Api::namespaced(client.clone(), namespace);

    resource_api
        .patch(resource_name, patch_params, &Patch::Apply(resource))
        .await?;

    Ok(())
}

pub async fn apply_resource_status<T, S>(
    client: &Client,
    status: S,
    resource_name: &str,
    namespace: &str,
    patch_params: &PatchParams,
) -> Result<S, kube::Error>
where
    S: Serialize,
    T: HasStatus<Status = S>
        + Default
        + Resource<Scope = NamespaceResourceScope>
        + Serialize
        + Clone
        + DeserializeOwned
        + Debug,
    <T as Resource>::DynamicType: Default,
{
    let resource_type_name = pretty_type_name::<T>();

    info!("Applying status for '{resource_name}' {resource_type_name}...",);

    let resource_api: Api<T> = Api::namespaced(client.clone(), namespace);
    let mut status_container = T::from_status(status);

    resource_api
        .patch_status(
            resource_name,
            patch_params,
            &Patch::Apply(&status_container),
        )
        .await?;

    Ok(status_container.status_mut().take().unwrap())
}

pub async fn apply_cluster_resource<T>(
    client: &Client,
    resource: &T,
    patch_params: &PatchParams,
) -> anyhow::Result<()>
where
    T: Resource<Scope = ClusterResourceScope> + Serialize + Clone + DeserializeOwned + Debug,
    <T as Resource>::DynamicType: Default,
{
    let resource_type_name = pretty_type_name::<T>();
    let resource_name = resource.meta().name.as_ref().unwrap();

    info!("Applying '{resource_name}' {resource_type_name} resource on the cluster...",);

    let resource_api: Api<T> = Api::all(client.clone());
    resource_api
        .patch(resource_name, patch_params, &Patch::Apply(resource))
        .await
        .context(format!(
            "Unable to create '{resource_name}' {resource_type_name} resource!"
        ))?;

    Ok(())
}

pub async fn apply_crd(
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

    info!("Applying {crd_name} ({crd_apiversions}) CRD...");

    let crd_api: Api<CustomResourceDefinition> = Api::all(client.clone());
    crd_api
        .patch(crd_name, patch_params, &Patch::Apply(crd))
        .await
        .context(format!(
            "Unable to create {crd_name} ({crd_apiversions}) CRD!"
        ))?;

    Ok(())
}

pub async fn remove_matching_resources<T>(
    client: &Client,
    list_params: &ListParams,
    delete_params: &DeleteParams,
) -> anyhow::Result<()>
where
    T: Resource<Scope = NamespaceResourceScope> + Serialize + Clone + DeserializeOwned + Debug,
    <T as Resource>::DynamicType: Default,
{
    let resource_type_name = pretty_type_name::<T>();
    let api: Api<T> = Api::all(client.clone());

    let resources = api.list_metadata(list_params).await.context(format!(
        "Couldn't retrieve {resource_type_name} from the cluster!"
    ))?;

    for resource in resources {
        let name = &resource
            .metadata
            .name
            .as_ref()
            .ok_or_else(|| anyhow!("Cluster returned a nameless {}!", resource_type_name))? // this shouldn't happen
            .as_str();
        let namespace = &resource
            .metadata
            .namespace
            .as_ref()
            .ok_or_else(|| anyhow!("Namespaced resource is missing a namespace!"))? // this shouldn't happen
            .as_str();

        info!("Removing '{name}' {resource_type_name} from the cluster...");

        let namespaced_api: Api<T> = Api::namespaced(client.clone(), namespace);

        namespaced_api
            .delete(name, delete_params)
            .await
            .context(format!(
                "Couldn't delete '{name}' {resource_type_name} (namespace: {namespace}) from the cluster!"
            ))?;
    }

    Ok(())
}

pub async fn remove_matching_cluster_resources<T>(
    client: &Client,
    list_params: &ListParams,
    delete_params: &DeleteParams,
) -> anyhow::Result<()>
where
    T: Resource<Scope = ClusterResourceScope> + Serialize + Clone + DeserializeOwned + Debug,
    <T as Resource>::DynamicType: Default,
{
    let resource_type_name = pretty_type_name::<T>();
    let api: Api<T> = Api::all(client.clone());

    let resources = api.list_metadata(list_params).await.context(format!(
        "Couldn't retrieve {resource_type_name} from the cluster!"
    ))?;

    for resource in resources {
        let name: &&str = &resource
            .metadata
            .name
            .as_ref()
            .ok_or_else(|| anyhow!("Cluster returned a nameless {}!", resource_type_name))? // this shouldn't happen
            .as_str();

        info!("Removing '{name}' {resource_type_name} from the cluster...");

        api.delete(name, delete_params).await.context(format!(
            "Couldn't delete '{resource_type_name}' from the cluster!"
        ))?;
    }

    Ok(())
}

pub async fn remove_cluster_resource<T>(
    client: &Client,
    resource_name: &str,
    delete_params: &DeleteParams,
) -> anyhow::Result<()>
where
    T: Resource<Scope = ClusterResourceScope> + Serialize + Clone + DeserializeOwned + Debug,
    <T as Resource>::DynamicType: Default,
{
    let resource_type_name = pretty_type_name::<T>();
    let resource_api: Api<T> = Api::all(client.clone());

    info!("Removing '{resource_name}' {resource_type_name} from the cluster...",);

    resource_api
        .delete(resource_name, delete_params)
        .await
        .context(format!("Couldn't delete {resource_name} from the cluster!"))?;

    Ok(())
}

pub async fn try_remove_cluster_resource<T>(
    client: &Client,
    resource_name: &str,
    delete_params: &DeleteParams,
) -> anyhow::Result<bool>
where
    T: Resource<Scope = ClusterResourceScope> + Serialize + Clone + DeserializeOwned + Debug,
    <T as Resource>::DynamicType: Default,
{
    let resource_api: Api<T> = Api::all(client.clone());

    info!(
        "Removing '{resource_name}' {} from the cluster...",
        pretty_type_name::<T>()
    );

    try_remove(&resource_api, resource_name, delete_params).await
}

pub async fn try_remove_resource<T>(
    client: &Client,
    resource_name: &str,
    namespace: &str,
    delete_params: &DeleteParams,
) -> anyhow::Result<bool>
where
    T: Resource<Scope = NamespaceResourceScope> + Serialize + Clone + DeserializeOwned + Debug,
    <T as Resource>::DynamicType: Default,
{
    let resource_api: Api<T> = Api::namespaced(client.clone(), namespace);

    try_remove(&resource_api, resource_name, delete_params).await
}

async fn try_remove<T>(
    resource_api: &Api<T>,
    resource_name: &str,
    delete_params: &DeleteParams,
) -> anyhow::Result<bool>
where
    T: Serialize + Clone + DeserializeOwned + Debug,
{
    info!(
        "Removing '{resource_name}' {} from the cluster...",
        pretty_type_name::<T>()
    );

    let delete_result = resource_api.delete(resource_name, delete_params).await;

    match delete_result {
        Ok(_) => Ok(true),
        Err(err) => {
            if let kube::Error::Api(api_err) = &err {
                if api_err.code == 404 {
                    return Ok(false);
                }
            }

            Err(err.into())
        }
    }
}
