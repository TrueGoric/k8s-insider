use std::{sync::Arc, time::Duration};

use k8s_insider_core::resources::crd::Tunnel;
use kube::runtime::controller::Action;

use self::{context::ReconcilerContext, error::ReconcilerError};

pub mod context;
pub mod error;

pub async fn reconcile_tunnel(
    object: Arc<Tunnel>,
    context: Arc<ReconcilerContext>,
) -> Result<Action, ReconcilerError> {
    Ok(Action::await_change())
}

pub fn reconcile_tunnel_error(
    object: Arc<Tunnel>,
    error: &ReconcilerError,
    context: Arc<ReconcilerContext>,
) -> Action {
    Action::requeue(Duration::from_secs(60))
}
