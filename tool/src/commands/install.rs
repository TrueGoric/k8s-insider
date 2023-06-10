use anyhow::{anyhow, Context};
use k8s_openapi::{
    api::{
        apps::v1::Deployment,
        core::v1::{ConfigMap, Service},
    },
    serde::de::DeserializeOwned,
};
use kube::{api::ListParams, Api, Client};
use log::{debug, warn, info};

use crate::{
    cli::{InstallArgs, GlobalArgs},
    kubernetes::create_client,
    resources::{
        get_common_release_listparams,
        release::{ReleaseInfo, ReleaseInfoBuilder},
    }, detectors::{detect_dns_service, detect_cluster_domain, detect_service_cidr, detect_pod_cidr},
};

pub async fn install(global_args: GlobalArgs, args: InstallArgs) -> anyhow::Result<()> {
    debug!("Loading configuration...");
    let client = create_client(&global_args.kube_config)
        .await
        .context("Couldn't initialize k8s API client!")?;

    let release_params = get_common_release_listparams(&args.release_name);

    debug!("Checking if there's another release with the same name in the cluster/namespace...");
    if check_if_release_exists(&release_params, &global_args.namespace, &client).await? {
        return Err(anyhow!(
            "Release {} already exists in the cluster!",
            args.release_name
        ));
    }

    debug!("Preparing release...");
    let release_info = prepare_release(args, &client).await?;

    Ok(())
}

async fn check_if_release_exists(
    release_params: &ListParams,
    namespace: &str,
    client: &Client,
) -> anyhow::Result<bool> {
    Ok(check_if_resource_exists::<Deployment>(
        &release_params,
        &Api::namespaced(client.clone(), namespace),
    )
    .await?
        || check_if_resource_exists::<Service>(
            &release_params,
            &Api::namespaced(client.clone(), namespace),
        )
        .await?
        || check_if_resource_exists::<ConfigMap>(
            &release_params,
            &Api::namespaced(client.clone(), namespace),
        )
        .await?)
}

async fn check_if_resource_exists<T: Clone + DeserializeOwned + core::fmt::Debug>(
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

async fn prepare_release(
    args: InstallArgs,
    client: &Client,
) -> anyhow::Result<ReleaseInfo> {
    let release_info = ReleaseInfoBuilder::default()
        .release_name({
            info!("Using release name: {}", args.release_name);
            args.release_name
        })
        .kube_dns(match args.kube_dns {
            Some(value) => {
                info!("Using DNS service IP: {value}");
                Some(value)
            },
            None => detect_dns_service(client).await?,
        })
        .service_domain(match args.service_domain {
            Some(value) => {
                info!("Using cluster domain: {value}");
                Some(value)
            },
            None => detect_cluster_domain(client).await?,
        })
        .service_cidr(match args.service_cidr {
            Some(value) => {
                info!("Using service CIDR: {value}");
                value
            },
            None => detect_service_cidr(client).await?,
        })
        .pod_cidr(match args.pod_cidr {
            Some(value) => {
                info!("Using pod CIDR: {value}");
                value
            },
            None => detect_pod_cidr(client).await?,
        })
        .build()?;
    
    debug!("{release_info:?}");

    Ok(release_info)
}
