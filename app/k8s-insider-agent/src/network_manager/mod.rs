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
use tokio::join;

use crate::network_manager::{allocations::sync_allocations, tunnel::start_tunnel_controller};

use self::{allocations::Ipv4AllocationsSync, reconciler::context::ReconcilerContext};

pub mod allocations;
pub mod reconciler;
pub mod tunnel;

pub const NETWORK_MANAGER_FIELD_MANAGER: &str = "k8s-insider-network-manager";

const NETWORK_NAME_ENV: &str = "KUBE_INSIDER_NETWORK_NAME";
const NETWORK_NAMESPACE_ENV: &str = "KUBE_INSIDER_NETWORK_NAMESPACE";

pub async fn main_network_manager(client: Client) {
    let controller_release = get_controller_release();
    let network_crd = get_ready_network_crd(&client).await;
    let router_release = get_router_release(&controller_release, &network_crd);

    let (allocations_ipv4, _) = sync_allocations(&client, &router_release)
        .await
        .unwrap_or_else(|error| {
            error!("Couldn't sync address allocations! {error:?}");
            exit(8)
        });

    let reconciler_context =
        get_reconciler_context(client, controller_release, router_release, network_crd, allocations_ipv4);

    let tunnel_controller = start_tunnel_controller(reconciler_context.into());

    join!(tunnel_controller);
}

fn get_reconciler_context(
    client: Client,
    controller_release: ControllerRelease,
    router_release: RouterRelease,
    owner: Network,
    allocations_ipv4: Option<Ipv4AllocationsSync>,
) -> ReconcilerContext {
    ReconcilerContext {
        controller_release,
        router_release,
        owner,
        client,
        allocations_ipv4,
    }
}

fn get_controller_release() -> ControllerRelease {
    match ControllerRelease::from_env() {
        Ok(release) => release,
        Err(error) => {
            error!("Couldn't retrieve controller release info! {error:?}");
            exit(7)
        }
    }
}

async fn get_ready_network_crd(client: &Client) -> Network {
    info!("Waiting for network to be ready...");

    let network_name = std::env::var(NETWORK_NAME_ENV).unwrap_or_else(|_| {
        error!("{NETWORK_NAME_ENV} must be set when running in network-manager mode!");
        exit(11)
    });

    let network_namespace = std::env::var(NETWORK_NAMESPACE_ENV).unwrap_or_else(|_| {
        error!("{NETWORK_NAMESPACE_ENV} must be set when running in network-manager mode!");
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

fn get_router_release(controller_release: &ControllerRelease, network: &Network) -> RouterRelease {
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
