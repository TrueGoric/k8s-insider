use kube::Client;
use tokio::join;

use crate::{
    controller::reconciler::context::ReconcilerContext, release::get_controller_release_from_env,
};

use self::{network::start_network_controller, node::start_node_reflector};

pub mod network;
pub mod node;
pub mod reconciler;

pub const CONTROLLER_FIELD_MANAGER: &str = "k8s-insider-controller";

pub async fn main_controller(client: Client) {
    let (reflector, nodes, ping) = start_node_reflector(&client).await;

    let reconciler_context = ReconcilerContext {
        release: get_controller_release_from_env(),
        client,
        nodes,
    };

    let controller = start_network_controller(reconciler_context.into(), ping);

    join!(reflector, controller);
}
