use std::fmt::Debug;

use k8s_insider_core::helpers::pretty_type_name;
use kube::{
    runtime::{
        controller::{Action, Error as ControllerError},
        reflector::ObjectRef,
        watcher::Error as WatcherError,
    },
    Resource,
};
use log::{error, info, warn};

pub fn handle_reconciliation_result<T, E>(
    result: Result<(ObjectRef<T>, Action), ControllerError<E, WatcherError>>,
) -> impl std::future::Future<Output = ()>
where
    T: Resource,
    E: Debug,
{
    let resource_name = pretty_type_name::<T>();

    match result {
        Ok(result) => info!(
            "Reconciled {} '{}' in '{}' namespace. Next action: {:?}",
            resource_name.to_lowercase(),
            result.0.name,
            result.0.namespace.as_deref().unwrap_or("---"),
            result.1
        ),
        Err(err) => match err {
            ControllerError::ObjectNotFound(_) => (), // Network is gone, our job here is done
            ControllerError::ReconcilerFailed(reconciler_error, with_obj) => {
                warn!(
                    "{} reconciliation failed for '{}' (namespace {}): {:#?}",
                    resource_name,
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

    std::future::ready(())
}
