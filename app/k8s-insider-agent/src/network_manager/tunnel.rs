use std::sync::Arc;

use futures::StreamExt;
use k8s_insider_core::{kubernetes::GetApi, resources::crd::v1alpha1::tunnel::Tunnel};
use kube::runtime::{watcher::Config, Controller};
use log::info;

use crate::{
    helpers::handle_reconciliation_result,
    network_manager::reconciler::tunnel::{reconcile_tunnel, reconcile_tunnel_error},
};

use super::reconciler::context::ReconcilerContext;

pub async fn start_tunnel_controller(context: Arc<ReconcilerContext>) {
    info!("Creating tunnel controller...");

    let watcher_config = Config::default();
    let controller = Controller::new(
        context
            .client
            .namespaced_api::<Tunnel>(&context.router_release.namespace),
        watcher_config.clone(),
    )
    .shutdown_on_signal()
    .run(reconcile_tunnel, reconcile_tunnel_error, context.clone())
    .for_each(handle_reconciliation_result::<Tunnel, _>);

    info!("Tunnel controller created!");

    controller.await;

    info!("Exiting tunnel controller!");
}
