use std::process::exit;

use k8s_insider_core::resources::controller::ControllerRelease;
use kube::Client;
use tokio::join;

use crate::controller::reconciler::context::ReconcilerContext;

use self::{network::start_network_controller, node::start_node_reflector};

pub mod network;
pub mod node;
pub mod reconciler;

pub const CONTROLLER_FIELD_MANAGER: &str = "k8s-insider-controller";

pub async fn main_controller(client: Client) {
    let (reflector, nodes, ping) = start_node_reflector(&client).await;

    let reconciler_context = ReconcilerContext {
        release: get_controller_release(),
        client,
        nodes,
    };

    let controller = start_network_controller(reconciler_context.into(), ping);

    join!(reflector, controller);
}

fn get_controller_release() -> ControllerRelease {
    match ControllerRelease::from_env() {
        Ok(release) => release,
        Err(error) => {
            log::error!("Couldn't retrieve release info! {error:?}");
            exit(7)
        }
    }
}
