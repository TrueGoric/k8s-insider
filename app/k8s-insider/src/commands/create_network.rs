use std::net::IpAddr;

use anyhow::anyhow;
use k8s_insider_core::{
    helpers::ApplyConditional,
    kubernetes::operations::{apply_resource, try_get_resource},
    resources::crd::v1alpha1::network::{Network, NetworkService, NetworkSpec},
};
use kube::{api::PatchParams, core::ObjectMeta, Client};
use log::{debug, info};

use crate::{
    cli::{CreateNetworkArgs, GlobalArgs, ServiceType},
    CLI_FIELD_MANAGER,
};

pub async fn create_network(
    global_args: GlobalArgs,
    args: CreateNetworkArgs,
    client: Client,
) -> anyhow::Result<()> {
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

    if existing_network.is_some() {
        info!("Network successfully updated!");
    }
    else {
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
