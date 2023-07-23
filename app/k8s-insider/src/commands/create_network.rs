use std::net::IpAddr;

use anyhow::{anyhow, Context};
use k8s_insider_core::{
    helpers::{AndIf, RequireMetadata},
    kubernetes::operations::{apply_resource, try_get_resource},
    resources::crd::v1alpha1::network::{Network, NetworkService, NetworkSpec},
};
use kube::{api::PatchParams, core::ObjectMeta};
use log::{debug, info, warn};

use crate::{
    cli::{CreateNetworkArgs, GlobalArgs, ServiceType},
    config::{network::NetworkIdentifier, ConfigContext},
    CLI_FIELD_MANAGER,
};

pub async fn create_network(
    global_args: GlobalArgs,
    args: CreateNetworkArgs,
    mut context: ConfigContext,
) -> anyhow::Result<()> {
    let client = context.create_client_with_default_context().await?;

    info!(
        "Creating '{}' network in '{}' namespace...",
        args.name, global_args.namespace
    );

    let existing_network =
        try_get_resource::<Network>(&client, &args.name, &global_args.namespace).await?;

    if existing_network.is_some() {
        if args.force {
            info!(
                "Network '{}' already exists! Force applying changes...",
                args.name
            );
        } else {
            info!(
                "Network '{}' already exists! Use --force to force apply changes...",
                args.name
            );

            return Ok(());
        }
    }

    let apply_params =
        PatchParams::apply(CLI_FIELD_MANAGER).and_if(args.dry_run, PatchParams::dry_run);
    let network_crd = create_network_crd(global_args.namespace, args)?;

    debug!("{network_crd:#?}");

    apply_resource(&client, &network_crd, &apply_params).await?;
    write_config(&network_crd, &mut context)?;

    if existing_network.is_some() {
        info!("Network successfully updated!");
    } else {
        info!("Network successfully created!");
    }

    Ok(())
}

fn create_network_crd(namespace: String, args: CreateNetworkArgs) -> anyhow::Result<Network> {
    Ok(Network {
        metadata: ObjectMeta {
            name: Some(args.name),
            namespace: Some(namespace),
            ..Default::default()
        },
        spec: NetworkSpec {
            peer_cidr: args.peer_cidr.into(),
            network_service: match args.service_type {
                ServiceType::None => None,
                ServiceType::ClusterIp => Some(NetworkService::ClusterIp {
                    ip: args.cluster_ip.map(|ip| ip.into()),
                }),
                ServiceType::NodePort => Some(NetworkService::NodePort {
                    cluster_ip: args.cluster_ip.map(|ip| ip.into()),
                    predefined_ips: args
                        .external_ip
                        .map(|ips| ips.iter().map(|ip| IpAddr::V4(ip.to_owned())).collect()),
                }),
                ServiceType::LoadBalancer => Some(NetworkService::LoadBalancer {
                    cluster_ip: args.cluster_ip.map(|ip| ip.into()),
                }),
                ServiceType::ExternalIp => Some(NetworkService::ExternalIp {
                    cluster_ip: args.cluster_ip.map(|ip| ip.into()),
                    ips: args
                        .external_ip
                        .map(|ips| ips.iter().map(|ip| IpAddr::V4(ip.to_owned())).collect())
                        .ok_or(anyhow!("--external-ip argument is mandatory when using service of type ExternalIp!"))?,
                }),
            },
            nat: None,
        },
        status: None,
    })
}

fn write_config(crd: &Network, context: &mut ConfigContext) -> anyhow::Result<()> {
    let kube_context = context.kube_context_name().to_owned();
    let name = crd
        .require_name_or(anyhow!("Missing Network CRD name!"))?
        .to_owned();
    let namespace = crd
        .require_namespace_or(anyhow!("Missing Network CRD namespace!"))?
        .to_owned();
    let entry = NetworkIdentifier::new(name, namespace, kube_context).into();
    let local_name = context
        .insider_config()
        .generate_config_network_name(&entry);

    if local_name != entry.id.name {
        warn!(
            "'{}' network is already present in the config, saving as '{}' instead.",
            entry.id.name, local_name
        );
    }

    context
        .insider_config_mut()
        .try_add_network(local_name, entry)?;

    context
        .insider_config()
        .save()
        .context("Couldn't save the configuration file!")?;

    Ok(())
}
