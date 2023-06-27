use std::sync::Arc;

use futures::StreamExt;
use k8s_insider_core::resources::crd::v1alpha1::network::Network;
use k8s_openapi::api::{
    apps::v1::Deployment,
    core::v1::{Secret, Service, ServiceAccount},
    rbac::v1::RoleBinding,
};
use kube::runtime::{
    controller::{Action, Error as ControllerError},
    reflector::ObjectRef,
    watcher::{Config, Error as WatcherError},
    Controller,
};
use log::{error, info, warn};

use crate::controller::reconciler::network::{reconcile_network, reconcile_network_error};

use super::reconciler::{context::ReconcilerContext, error::ReconcilerError};

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
        .for_each(handle_reconciliation_result);

    info!("Network controller created!");

    controller.await
}

async fn handle_reconciliation_result(
    result: Result<(ObjectRef<Network>, Action), ControllerError<ReconcilerError, WatcherError>>,
) {
    match result {
        Ok(result) => info!(
            "Reconciled network '{}' in '{}' namespace. Next action: {:?}",
            result.0.name,
            result.0.namespace.as_deref().unwrap_or("---"),
            result.1
        ),
        Err(err) => match err {
            ControllerError::ObjectNotFound(_) => (), // Network is gone, our job here is done
            ControllerError::ReconcilerFailed(reconciler_error, with_obj) => {
                warn!(
                    "Network reconciliation failed for '{}' (namespace {}): {:#?}",
                    with_obj.name,
                    with_obj.namespace.as_deref().unwrap_or("---"),
                    reconciler_error
                )
            }
            ControllerError::QueueError(watcher_err) => {
                error!("Watcher has failed! {watcher_err:#?}")
            }
        },
    }
}
