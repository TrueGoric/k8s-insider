use std::sync::Arc;

use futures::StreamExt;
use k8s_insider_core::resources::crd::v1alpha1::network::Network;
use k8s_openapi::api::{
    apps::v1::Deployment,
    core::v1::{Secret, Service, ServiceAccount},
    rbac::v1::RoleBinding,
};
use kube::runtime::{watcher::Config, Controller};
use log::{info, warn};

use crate::controller::reconciler::network::{reconcile_network, reconcile_network_error};

use super::reconciler::context::ReconcilerContext;

pub async fn start_network_controller(context: &Arc<ReconcilerContext>) {
    info!("Creating network controller...");

    let watcher_config = Config::default();
    let controller = Controller::new(context.global_api::<Network>(), watcher_config.clone())
        .owns(context.global_api::<Secret>(), watcher_config.clone())
        .owns(context.global_api::<Deployment>(), watcher_config.clone())
        .owns(context.global_api::<Service>(), watcher_config.clone())
        .owns(context.global_api::<RoleBinding>(), watcher_config.clone())
        .owns(
            context.global_api::<ServiceAccount>(),
            watcher_config.clone(),
        )
        .shutdown_on_signal()
        .run(reconcile_network, reconcile_network_error, context.clone())
        .for_each(|tunnel| async move {
            match tunnel {
                Ok(o) => info!("Reconciled network {:?}", o),
                Err(e) => warn!("Network reconciliation failed: {:#?}", e), // this should be something more explicit and readable perhaps
            }
        });

    info!("Network controller created!");

    controller.await
}
