use anyhow::{anyhow, Context};
use k8s_insider_core::{
    detectors::{detect_cluster_domain, detect_dns_service, detect_pod_cidr, detect_service_cidr},
    kubernetes::operations::{
        check_if_resource_exists, create_cluster_resource, create_namespace_if_not_exists,
        create_resource,
    },
    resources::{
        controller::{ControllerRelease, ControllerReleaseBuilder},
        crd::v1alpha1::create_v1alpha1_crds,
        labels::get_controller_listparams,
    },
    FIELD_MANAGER,
};
use k8s_openapi::api::apps::v1::Deployment;
use kube::{
    api::{ListParams, PatchParams},
    Api, Client,
};
use log::{debug, info, warn};

use crate::cli::{GlobalArgs, InstallArgs};

pub async fn install(
    global_args: &GlobalArgs,
    args: &InstallArgs,
    client: &Client,
) -> anyhow::Result<()> {
    info!(
        "Installing k8s-insider into '{}' namespace...",
        global_args.namespace
    );

    let release_params = get_controller_listparams();

    debug!("Checking if k8s-insider is already installed...");
    if check_if_release_exists(&release_params, &global_args.namespace, client).await? {
        if args.force {
            warn!(
                "k8s-insider is already installed in the namespace '{}', force deploying...",
                global_args.namespace
            );
        } else {
            return Err(anyhow!(
                "k8s-insider is already installed in the namespace '{}'!",
                global_args.namespace
            ));
        }
    }

    debug!("Preparing release...");
    let release_info = prepare_release(global_args, args, client).await?;

    create_v1alpha1_crds(client, args.dry_run).await?;
    deploy_release(&release_info, client, args.dry_run).await?;

    info!("Successfully deployed k8s-insider!");

    Ok(())
}

async fn check_if_release_exists(
    release_params: &ListParams,
    namespace: &str,
    client: &Client,
) -> anyhow::Result<bool> {
    check_if_resource_exists::<Deployment>(
        release_params,
        &Api::namespaced(client.clone(), namespace),
    )
    .await
}

async fn prepare_release(
    global_args: &GlobalArgs,
    args: &InstallArgs,
    client: &Client,
) -> anyhow::Result<ControllerRelease> {
    let release_info = ControllerReleaseBuilder::default()
        .namespace({
            info!("Using release namespace: {}", global_args.namespace);
            global_args.namespace.clone()
        })
        .service_cidr(match &args.service_cidr {
            Some(value) => {
                info!("Using service CIDR: {value}");
                value.trunc().into()
            }
            None => detect_service_cidr(client).await?,
        })
        .pod_cidr(match &args.pod_cidr {
            Some(value) => {
                info!("Using pod CIDR: {value}");
                value.trunc().into()
            }
            None => detect_pod_cidr(client).await?,
        })
        .kube_dns(match &args.kube_dns {
            Some(value) => {
                info!("Using DNS service IP: {value}");
                Some(value.parse()?)
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
        .controller_image_name({
            info!("Using controller image: {}", args.controller_image);
            args.controller_image.clone()
        })
        .tunnel_image_name({
            info!("Using router image: {}", args.router_image);
            args.router_image.clone()
        })
        .build()?;

    debug!("{release_info:#?}");

    Ok(release_info)
}

async fn deploy_release(
    release: &ControllerRelease,
    client: &Client,
    dry_run: bool,
) -> anyhow::Result<()> {
    let namespace = &release.namespace;
    let serviceaccount = release.generate_controller_service_account();
    let controller_clusterrole = release.generate_controller_cluster_role();
    let controller_clusterrole_binding = release
        .generate_controller_cluster_role_binding(&controller_clusterrole, &serviceaccount)
        .context("Couldn't generate controller cluster role binding!")?;
    let router_clusterrole = release.generate_router_clusterrole();
    let configmap = release.generate_configmap();
    let deployment = release
        .generate_deployment(&configmap, &serviceaccount)
        .context("Couldn't generate controller deployment!")?;

    debug!("{serviceaccount:#?}");
    debug!("{configmap:#?}");
    debug!("{deployment:#?}");

    let patch_params = PatchParams {
        dry_run,
        field_manager: Some(FIELD_MANAGER.to_owned()),
        ..Default::default()
    };

    create_namespace_if_not_exists(client, &patch_params, namespace).await?;
    create_resource(client, &serviceaccount, &patch_params).await?;
    create_cluster_resource(client, &controller_clusterrole, &patch_params).await?;
    create_cluster_resource(client, &controller_clusterrole_binding, &patch_params).await?;
    create_cluster_resource(client, &router_clusterrole, &patch_params).await?;
    create_resource(client, &deployment, &patch_params).await?;
    create_resource(client, &configmap, &patch_params).await?;

    Ok(())
}
