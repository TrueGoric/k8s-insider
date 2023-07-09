use std::{pin::pin, process::exit};

use futures::StreamExt;
use k8s_insider_core::{
    kubernetes::operations::watch_resource,
    resources::{
        controller::ControllerRelease,
        crd::v1alpha1::network::Network,
        router::{RouterRelease, RouterReleaseBuilder},
    },
};
use kube::Client;
use log::{error, info};

pub const NETWORK_NAME_ENV: &str = "KUBE_INSIDER_NETWORK_NAME";
pub const NETWORK_NAMESPACE_ENV: &str = "KUBE_INSIDER_NETWORK_NAMESPACE";

pub async fn get_ready_network_crd(client: &Client) -> Network {
    info!("Waiting for network to be ready...");

    let network_name = std::env::var(NETWORK_NAME_ENV).unwrap_or_else(|_| {
        error!("{NETWORK_NAME_ENV} must be set!");
        exit(11)
    });

    let network_namespace = std::env::var(NETWORK_NAMESPACE_ENV).unwrap_or_else(|_| {
        error!("{NETWORK_NAMESPACE_ENV} must be set!");
        exit(12)
    });

    let mut network_watch = pin!(watch_resource::<Network>(
        client,
        &network_name,
        &network_namespace
    ));

    while let Some(network) = network_watch.next().await {
        let network = network.unwrap_or_else(|err| {
            error!("Couldn't retrieve the Network CRD! {err:?}");
            exit(13)
        });

        match network {
            Some(network) => return network,
            None => continue,
        }
    }

    error!("{network_name} Network CRD was not detected on the cluster!");
    exit(14)
}

pub fn get_router_release(controller_release: &ControllerRelease, network: &Network) -> RouterRelease {
    RouterReleaseBuilder::default()
        .with_controller(controller_release)
        .with_network_crd(network)
        .unwrap_or_else(|err| {
            error!("Invalid network CRD data! {err:?}");
            exit(21)
        })
        .build()
        .unwrap_or_else(|err| {
            error!("Couldn't construct router release info! {err:?}");
            exit(22)
        })
        .validated()
        .unwrap_or_else(|err| {
            error!("Couldn't validate router release info! {err:?}");
            exit(23)
        })
}
