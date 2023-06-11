use anyhow::{anyhow, Context};
use k8s_openapi::{
    api::{apps::v1::Deployment, core::v1::ConfigMap},
    serde::de::DeserializeOwned,
};
use kube::{
    api::{ListParams, Patch, PatchParams},
    Api, Client,
};
use log::{debug, info, warn};

use crate::{
    cli::{GlobalArgs, InstallArgs},
    detectors::{detect_cluster_domain, detect_dns_service, detect_pod_cidr, detect_service_cidr},
    resources::{
        configmap::generate_configmap,
        deployment::generate_deployment,
        get_release_listparams,
        namespace::create_namespace_if_not_exists,
        release::{Release, ReleaseBuilder},
    },
};

const FIELD_MANAGER: &str = "k8s-insider";

pub async fn install(
    global_args: &GlobalArgs,
    args: &InstallArgs,
    client: &Client,
) -> anyhow::Result<()> {
    info!("Installing release '{}'...", args.release_name);

    let release_params = get_release_listparams(&args.release_name);

    debug!("Checking if there's another release with the same name in the cluster/namespace...");
    if check_if_release_exists(&release_params, &global_args.namespace, &client).await? {
        if args.force {
            warn!(
                "Release {} already exists in the cluster, force deploying...",
                args.release_name
            );
        } else {
            return Err(anyhow!(
                "Release {} already exists in the cluster!",
                args.release_name
            ));
        }
    }

    debug!("Preparing release...");
    let release_info = prepare_release(global_args, args, &client).await?;
    let configmap = generate_configmap(&release_info);
    let configmap_name = configmap.metadata.name.as_ref().unwrap();
    let deployment = generate_deployment(&release_info, &configmap.metadata.name.as_ref().unwrap());
    let deployment_name = deployment.metadata.name.as_ref().unwrap();

    debug!("{configmap:#?}");
    debug!("{deployment:#?}");

    let configmap_api: Api<ConfigMap> = Api::namespaced(client.clone(), &global_args.namespace);
    let deployment_api: Api<Deployment> = Api::namespaced(client.clone(), &global_args.namespace);
    let patch_params = PatchParams {
        dry_run: args.dry_run,
        field_manager: Some(FIELD_MANAGER.to_owned()),
        ..Default::default()
    };

    info!(
        "Ensuring the namespace '{}' is created...",
        release_info.release_namespace
    );
    create_namespace_if_not_exists(&client, &patch_params, &release_info.release_namespace).await?;

    info!("Creating the configmap on the cluster...");
    configmap_api
        .patch(configmap_name, &patch_params, &Patch::Apply(&configmap))
        .await
        .context("Unable to create the configmap!")?;

    info!("Creating the deployment on the cluster...");
    deployment_api
        .patch(deployment_name, &patch_params, &Patch::Apply(&deployment))
        .await
        .context("Unable to create the deployment!")?;

    info!(
        "Successfully deployed '{}' release!",
        release_info.release_name
    );

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
    global_args: &GlobalArgs,
    args: &InstallArgs,
    client: &Client,
) -> anyhow::Result<Release> {
    let release_info = ReleaseBuilder::default()
        .release_name({
            info!("Using release name: {}", args.release_name);
            args.release_name.clone()
        })
        .release_namespace({
            info!("Using release namespace: {}", global_args.namespace);
            global_args.namespace.clone()
        })
        .image_name({
            info!("Using image name: {}", args.image_name);
            args.image_name.clone()
        })
        .kube_dns(match &args.kube_dns {
            Some(value) => {
                info!("Using DNS service IP: {value}");
                Some(value.clone())
            }
            None => detect_dns_service(client).await?,
        })
        .service_domain(match &args.service_domain {
            Some(value) => {
                info!("Using cluster domain: {value}");
                Some(value.clone())
            }
            None => detect_cluster_domain(client).await?,
        })
        .service_cidr(match &args.service_cidr {
            Some(value) => {
                info!("Using service CIDR: {value}");
                value.clone()
            }
            None => detect_service_cidr(client).await?,
        })
        .pod_cidr(match &args.pod_cidr {
            Some(value) => {
                info!("Using pod CIDR: {value}");
                value.clone()
            }
            None => detect_pod_cidr(client).await?,
        })
        .build()?;

    debug!("{release_info:#?}");

    Ok(release_info)
}
