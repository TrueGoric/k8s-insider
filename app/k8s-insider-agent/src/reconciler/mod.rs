use std::{sync::Arc, time::Duration};

use k8s_insider_core::resources::crd::v1alpha1::tunnel::{Tunnel, TunnelState, TunnelStatus};
use kube::{
    api::{Patch, PatchParams},
    runtime::controller::Action,
    Resource,
};

use self::{context::ReconcilerContext, error::ReconcilerError};

pub mod context;
pub mod error;

const RECONCILE_REQUEUE_SECS: u64 = 60 * 5;
const ERROR_REQUEUE_SECS: u64 = 1;

pub async fn reconcile_tunnel(
    object: Arc<Tunnel>,
    context: Arc<ReconcilerContext>,
) -> Result<Action, ReconcilerError> {
    let tunnel_name = object.metadata.name.as_ref().unwrap();
    let mut status = ensure_status(tunnel_name, &object, &context).await?;
    let owner_ref = object.controller_owner_ref(&()).unwrap();

    Ok(Action::requeue(Duration::from_secs(RECONCILE_REQUEUE_SECS)))
}

async fn ensure_status(
    tunnel_name: &str,
    object: &Tunnel,
    context: &ReconcilerContext,
) -> Result<TunnelStatus, ReconcilerError> {
    let status = object.status.to_owned().unwrap_or_else(|| TunnelStatus {
        state: TunnelState::Creating,
        ..Default::default()
    });

    context
        .tunnel_api
        .patch_status(tunnel_name, &PatchParams::default(), &Patch::Apply(&status))
        .await
        .map_err(ReconcilerError::PatchError)?;

    Ok(status)
}

pub fn reconcile_tunnel_error(
    _object: Arc<Tunnel>,
    _error: &ReconcilerError,
    _context: Arc<ReconcilerContext>,
) -> Action {
    Action::requeue(Duration::from_secs(ERROR_REQUEUE_SECS))
}
