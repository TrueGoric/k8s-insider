use anyhow::{anyhow, Context};
use k8s_insider_core::{
    detectors::{detect_cluster_domain, detect_dns_service, detect_pod_cidr, detect_service_cidr},
    helpers::{AndIf, ErrLogger},
    kubernetes::operations::{
        apply_cluster_resource, apply_resource, create_namespace_if_not_exists, list_resources,
    },
    resources::{
        controller::ControllerRelease, crd::v1alpha1::create_v1alpha1_crds,
        labels::get_controller_listparams,
    },
};
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{
    api::{ListParams, PatchParams},
    Client,
};
use log::{debug, info, warn};

use crate::{
    cli::{GlobalArgs, InstallArgs},
    context::ConfigContext,
    CLI_FIELD_MANAGER,
};

pub async fn install(
    global_args: GlobalArgs,
    args: InstallArgs,
    context: ConfigContext,
) -> anyhow::Result<()> {
    let client = context.create_client_with_default_context().await?;

    if args.upgrade {
        info!(
            "Upgrading cluster's k8s-insider release in '{}' namespace...",
            global_args.namespace
        );
    } else {
        info!(
            "Installing k8s-insider into '{}' namespace...",
            global_args.namespace
        );
    }

    let no_crds = args.no_crds;
    let dry_run = args.dry_run;

    debug!("Checking if k8s-insider is already installed...");

    let release_params = get_controller_listparams();
    let installed_release =
        try_get_installed_release(&release_params, &global_args.namespace, &client).await?;

    if let Some(release) = installed_release {
        if args.force {
            warn!(
                "k8s-insider is already installed in the namespace '{}', force deploying...",
                global_args.namespace
            );
        } else if args.upgrade {
            perform_upgrade(release, args, client, dry_run, no_crds).await?;
        } else {
            return Err(anyhow!(
                "k8s-insider is already installed in the namespace '{}'!",
                global_args.namespace
            ));
        }
    } else {
        perform_installation(global_args, args, client, dry_run, no_crds).await?;
    }

    Ok(())
}

async fn perform_installation(
    global_args: GlobalArgs,
    args: InstallArgs,
    client: Client,
    dry_run: bool,
    no_crds: bool,
) -> anyhow::Result<()> {
    debug!("Preparing release...");

    let release_info = prepare_release(global_args.namespace, args, &client).await?;

    apply_release(dry_run, no_crds, client, release_info).await?;

    info!("Successfully deployed k8s-insider!");

    Ok(())
}

async fn perform_upgrade(
    current_release: ControllerRelease,
    args: InstallArgs,
    client: Client,
    dry_run: bool,
    no_crds: bool,
) -> anyhow::Result<()> {
    // might wanna make this a little bit more elegant when a breaking change is introduced,
    // but KISS and YAGNI and stuff

    debug!("Preparing upgrade...");

    let release_info = prepare_upgrade(current_release, args)?;

    apply_release(dry_run, no_crds, client, release_info).await?;

    info!("Successfully upgraded k8s-insider!");

    Ok(())
}

async fn apply_release(
    dry_run: bool,
    no_crds: bool,
    client: Client,
    release_info: ControllerRelease,
) -> Result<(), anyhow::Error> {
    let apply_params = PatchParams::apply(CLI_FIELD_MANAGER).and_if(dry_run, |s| s.dry_run());

    if no_crds {
        info!("Skipping CRD deployment...");
    } else {
        create_v1alpha1_crds(&client, &apply_params).await?;
    }

    deploy_release(release_info, &client, &apply_params).await?;

    Ok(())
}

async fn try_get_installed_release(
    release_params: &ListParams,
    namespace: &str,
    client: &Client,
) -> anyhow::Result<Option<ControllerRelease>> {
    let configmap = list_resources::<ConfigMap>(client, namespace, release_params).await?;

    if configmap.is_empty() {
        return Ok(None);
    }

    let release = ControllerRelease::from_configmap(&configmap[0])
        .context("Couldn't parse release ConfigMap!")?;

    Ok(Some(release))
}

async fn prepare_release(
    namespace: String,
    args: InstallArgs,
    client: &Client,
) -> anyhow::Result<ControllerRelease> {
    let service_cidr = match &args.service_cidr {
        Some(value) => {
            info!("Using service CIDR: {value}");
            Ok(value.trunc().into())
        }
        None => detect_service_cidr(client).await.log_error().map_err(|_| {
            anyhow!("Couldn't autodetect some parameters! Try passing them manually.")
        }),
    };

    let pod_cidr = match &args.pod_cidr {
        Some(value) => {
            info!("Using pod CIDR: {value}");
            Ok(value.trunc().into())
        }
        None => detect_pod_cidr(client).await.log_error().map_err(|_| {
            anyhow!("Couldn't autodetect some parameters! Try passing them manually.")
        }),
    };

    let kube_dns = match &args.kube_dns {
        Some(value) => {
            info!("Using DNS service IP: {value}");
            Ok(Some(value.parse()?))
        }
        None => detect_dns_service(client).await.log_error().map_err(|_| {
            anyhow!("Couldn't autodetect some parameters! Try passing them manually.")
        }),
    };

    let service_domain = match &args.service_domain {
        Some(value) => {
            info!("Using cluster domain: {value}");
            Ok(Some(value.clone()))
        }
        None => detect_cluster_domain(client)
            .await
            .log_error()
            .map_err(|_| {
                anyhow!("Couldn't autodetect some parameters! Try passing them manually.")
            }),
    };

    info!(
        "Using controller image: {}:{}",
        args.controller_image, args.controller_image_tag
    );
    let controller_image_name = args.controller_image.clone();
    let controller_image_tag = args.controller_image_tag.clone();

    info!(
        "Using network manager image: {}:{}",
        args.network_manager_image, args.network_manager_image_tag
    );
    let network_manager_image_name = args.network_manager_image.clone();
    let network_manager_image_tag = args.network_manager_image_tag.clone();

    info!(
        "Using router image: {}:{}",
        args.router_image, args.router_image_tag
    );
    let router_image_name = args.router_image.clone();
    let router_image_tag = args.router_image_tag.clone();

    let release_info = ControllerRelease {
        namespace: {
            info!("Using release namespace: {}", namespace);
            namespace
        },
        service_cidr: service_cidr?,
        pod_cidr: pod_cidr?,
        kube_dns: kube_dns?,
        service_domain: service_domain?,
        controller_image_name,
        controller_image_tag,
        network_manager_image_name,
        network_manager_image_tag,
        router_image_name,
        router_image_tag,
    };

    debug!("{release_info:#?}");

    Ok(release_info)
}

fn prepare_upgrade(
    mut current_release: ControllerRelease,
    args: InstallArgs,
) -> anyhow::Result<ControllerRelease> {
    info!(
        "Setting controller image: {}:{}",
        args.controller_image, args.controller_image_tag
    );
    current_release.controller_image_name = args.controller_image;
    current_release.controller_image_tag = args.controller_image_tag;

    info!(
        "Setting network manager image: {}:{}",
        args.network_manager_image, args.network_manager_image_tag
    );
    current_release.network_manager_image_name = args.network_manager_image;
    current_release.network_manager_image_tag = args.network_manager_image_tag;

    info!(
        "Setting router image: {}:{}",
        args.router_image, args.router_image_tag
    );
    current_release.router_image_name = args.router_image;
    current_release.router_image_tag = args.router_image_tag;

    Ok(current_release)
}

async fn deploy_release(
    release: ControllerRelease,
    client: &Client,
    apply_params: &PatchParams,
) -> anyhow::Result<()> {
    let serviceaccount = release.generate_controller_service_account();
    let controller_clusterrole = release.generate_controller_clusterrole();
    let network_manager_clusterrole = release.generate_network_manager_clusterrole();
    let router_clusterrole = release.generate_router_clusterrole();
    let configmap = release.generate_configmap();
    let controller_clusterrole_binding = release
        .generate_controller_cluster_role_binding(&controller_clusterrole, &serviceaccount)
        .context("Couldn't generate controller cluster role binding!")?;
    let deployment = release
        .generate_deployment(&configmap, &serviceaccount)
        .context("Couldn't generate controller deployment!")?;

    create_namespace_if_not_exists(client, apply_params, &release.namespace).await?;
    apply_cluster_resource(client, &controller_clusterrole, apply_params).await?;
    apply_cluster_resource(client, &network_manager_clusterrole, apply_params).await?;
    apply_cluster_resource(client, &router_clusterrole, apply_params).await?;
    apply_resource(client, &serviceaccount, apply_params).await?;
    apply_cluster_resource(client, &controller_clusterrole_binding, apply_params).await?;
    apply_resource(client, &deployment, apply_params).await?;
    apply_resource(client, &configmap, apply_params).await?;

    Ok(())
}
