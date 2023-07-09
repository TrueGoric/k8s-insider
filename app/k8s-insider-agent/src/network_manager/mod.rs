use std::process::exit;

use k8s_insider_core::resources::controller::ControllerRelease;
use kube::Client;
use log::error;
use tokio::join;

use crate::{
    network_manager::{allocations::sync_allocations, tunnel::start_tunnel_controller},
    release::{get_ready_network_crd, get_router_release},
};

use self::reconciler::context::ReconcilerContext;

pub mod allocations;
pub mod reconciler;
pub mod tunnel;

pub const NETWORK_MANAGER_FIELD_MANAGER: &str = "k8s-insider-network-manager";

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

    let reconciler_context = ReconcilerContext {
        controller_release,
        router_release,
        owner: network_crd,
        client,
        allocations_ipv4,
    };

    let tunnel_controller = start_tunnel_controller(reconciler_context.into());

    join!(tunnel_controller);
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
