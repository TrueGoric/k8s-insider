use anyhow::anyhow;
use k8s_insider_core::{
    detectors::{detect_cluster_domain, detect_dns_service, detect_pod_cidr, detect_service_cidr},
    kubernetes::operations::{
        check_if_resource_exists, create_namespace_if_not_exists, create_resource,
    },
    resources::{
        labels::get_release_listparams,
        release::{Release, ReleaseBuilder, ReleaseService},
    },
};
use k8s_openapi::api::apps::v1::Deployment;
use kube::{
    api::{ListParams, PatchParams},
    Api, Client,
};
use log::{debug, info, warn};

use crate::cli::{GlobalArgs, InstallArgs, ServiceType};

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
    let configmap = release_info.generate_configmap();
    let deployment = release_info.generate_deployment(&configmap);
    let service = release_info.generate_service(extract_port_name(&deployment));

    debug!("{configmap:#?}");
    debug!("{deployment:#?}");
    debug!("{service:#?}");

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

    create_resource(client, &global_args.namespace, &deployment, &patch_params).await?;
    create_resource(client, &global_args.namespace, &configmap, &patch_params).await?;

    if let Some(service) = service {
        create_resource(client, &global_args.namespace, &service, &patch_params).await?;
    }

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

fn extract_port_name(deployment: &Deployment) -> &str {
    deployment
        .spec
        .as_ref()
        .unwrap()
        .template
        .spec
        .as_ref()
        .unwrap()
        .containers
        .first()
        .unwrap()
        .ports
        .as_ref()
        .unwrap()
        .first()
        .unwrap()
        .name
        .as_ref()
        .unwrap() // ┌(˘⌣˘)ʃ
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
        .service(match &args.service_type {
            ServiceType::None => ReleaseService::None,
            ServiceType::NodePort => ReleaseService::NodePort {
                predefined_ips: args.external_ip.clone()
            },
            ServiceType::LoadBalancer => ReleaseService::LoadBalancer,
            ServiceType::ExternalIp => ReleaseService::ExternalIp {
                ip: args.external_ip
                    .as_ref()
                    .ok_or_else(|| anyhow!("--external-ip argument is mandatory when using service of type ExternalIp!"))?
                    .clone()
                },
        })
        .build()?;

    debug!("{release_info:#?}");

    Ok(release_info)
}
