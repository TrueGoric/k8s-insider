use anyhow::anyhow;
use k8s_insider_core::{
    detectors::{detect_cluster_domain, detect_dns_service, detect_pod_cidr, detect_service_cidr},
    kubernetes::operations::{
        check_if_resource_exists, create_namespace_if_not_exists, create_resource,
    },
    resources::{
        labels::get_tunnel_listparams,
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
    info!("Installing k8s-insider into '{}' namespace...", global_args.namespace);

    let release_params = get_tunnel_listparams();

    debug!("Checking if k8s-insider is already installed...");
    if check_if_release_exists(&release_params, &global_args.namespace, &client).await? {
        if args.force {
            warn!(
                "k8s-insider was already installed in the namespace '{}', force deploying...",
                global_args.namespace
            );
        } else {
            return Err(anyhow!(
                "k8s-insider was already installed in the namespace '{}'!",
                global_args.namespace
            ));
        }
    }

    debug!("Preparing release...");
    let release_info = prepare_release(global_args, args, &client).await?;
    let configmap = release_info.generate_configmap();
    let deployment = release_info.generate_tunnel_deployment(&configmap);
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
        release_info.namespace
    );
    create_namespace_if_not_exists(&client, &patch_params, &release_info.namespace).await?;

    create_resource(client, &global_args.namespace, &deployment, &patch_params).await?;
    create_resource(client, &global_args.namespace, &configmap, &patch_params).await?;

    if let Some(service) = service {
        create_resource(client, &global_args.namespace, &service, &patch_params).await?;
    }

    info!(
        "Successfully deployed k8s-insider!"
    );

    Ok(())
}

async fn check_if_release_exists(
    tunnel_params: &ListParams,
    namespace: &str,
    client: &Client,
) -> anyhow::Result<bool> {
    Ok(check_if_resource_exists::<Deployment>(
        &tunnel_params,
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
        .namespace({
            info!("Using release namespace: {}", global_args.namespace);
            global_args.namespace.clone()
        })
        .agent_image_name({
            info!("Using agent image name: {}", args.agent_image_name);
            args.agent_image_name.clone()
        })
        .tunnel_image_name({
            info!("Using tunnel image name: {}", args.tunnel_image_name);
            args.tunnel_image_name.clone()
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
