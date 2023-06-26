use std::net::IpAddr;

use anyhow::anyhow;
use k8s_insider_core::{resources::crd::v1alpha1::network::{Network, NetworkService, NetworkSpec}, kubernetes::operations::apply_resource, FIELD_MANAGER};
use kube::{core::ObjectMeta, Client, api::PatchParams};
use log::{info, debug};

use crate::cli::{CreateNetworkArgs, GlobalArgs, ServiceType};

pub async fn create_network(
    global_args: GlobalArgs,
    args: CreateNetworkArgs,
    client: Client,
) -> anyhow::Result<()> {
    info!(
        "Creating '{}' network into '{}' namespace...",
        args.name, global_args.namespace
    );

    let dry_run = args.dry_run;
    let network_crd = create_network_crd(global_args.namespace, args)?;
    let mut patch_params = PatchParams::apply(FIELD_MANAGER);

    if dry_run {
        patch_params = patch_params.dry_run();
    }

    debug!("{network_crd:#?}");

    apply_resource(&client, &network_crd, &patch_params).await?;

    info!("Network successfully created!");

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
