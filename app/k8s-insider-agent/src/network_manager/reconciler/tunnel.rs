use std::{sync::Arc, time::Duration};

use k8s_insider_core::{
    helpers::RequireMetadata,
    ip::addrpair::{DualStackTryGet, IpAddrPair},
    kubernetes::{operations::apply_resource_status, GetApi},
    resources::crd::v1alpha1::tunnel::{Tunnel, TunnelState, TunnelStatus},
    wireguard::keys::WgKey,
};
use kube::{
    api::PatchParams,
    runtime::{
        controller::Action,
        finalizer::{finalizer, Error as FinalizerError, Event as FinalizerEvent},
    },
    CustomResourceExt,
};

use crate::network_manager::{allocations::AllocationsError, NETWORK_MANAGER_FIELD_MANAGER};

use super::{context::ReconcilerContext, error::ReconcilerError};

const RECONCILE_REQUEUE_SECS: u64 = 60 * 5;
const USER_ERROR_REQUEUE_SECS: u64 = 60 * 5;
const ERROR_REQUEUE_SECS: u64 = 10;

pub async fn reconcile_tunnel(
    object: Arc<Tunnel>,
    context: Arc<ReconcilerContext>,
) -> Result<Action, FinalizerError<ReconcilerError>> {
    let tunnel_api = context
        .client
        .namespaced_api(&context.router_release.namespace);
    let finalizer_name = format!("{}/cleanup", Tunnel::crd_name());

    finalizer(&tunnel_api, &finalizer_name, object, |event| async {
        match event {
            FinalizerEvent::Apply(tunnel) => try_reconcile(&tunnel, &context).await,
            FinalizerEvent::Cleanup(tunnel) => cleanup(&tunnel, &context).await,
        }
    })
    .await
}

pub fn reconcile_tunnel_error(
    _object: Arc<Tunnel>,
    error: &FinalizerError<ReconcilerError>,
    _context: Arc<ReconcilerContext>,
) -> Action {
    Action::requeue(match error {
        FinalizerError::ApplyFailed(ReconcilerError::Ipv4AllocationError(err)) => match err {
            AllocationsError::WgKeyConflict(_) => Duration::from_secs(USER_ERROR_REQUEUE_SECS),
            AllocationsError::IpConflict(_) => Duration::from_secs(USER_ERROR_REQUEUE_SECS),
            AllocationsError::RangeExhausted => Duration::from_secs(ERROR_REQUEUE_SECS),
            AllocationsError::IpOutOfRange(_) => Duration::from_secs(USER_ERROR_REQUEUE_SECS),
        },
        _ => Duration::from_secs(ERROR_REQUEUE_SECS),
    })
}

async fn try_reconcile(
    object: &Tunnel,
    context: &ReconcilerContext,
) -> Result<Action, ReconcilerError> {
    let reconcile_result = reconcile(object, context).await;

    match reconcile_result {
        Ok(action) => Ok(action),
        Err(error) => {
            let state = get_error_state(&error);
            let status = TunnelStatus {
                state,
                ..Default::default()
            };

            let _ = apply_resource_status::<Tunnel, TunnelStatus>(
                &context.client,
                status,
                object.require_name_or(ReconcilerError::MissingObjectMetadata)?,
                object.require_namespace_or(ReconcilerError::MissingObjectMetadata)?,
                &PatchParams::apply(NETWORK_MANAGER_FIELD_MANAGER),
            )
            .await;

            Err(error)
        }
    }
}

fn get_error_state(error: &ReconcilerError) -> TunnelState {
    match error {
        ReconcilerError::Ipv4AllocationError(err) => match err {
            AllocationsError::WgKeyConflict(_) => TunnelState::ErrorPublicKeyConflict,
            AllocationsError::IpConflict(_) => TunnelState::ErrorIpAlreadyInUse,
            AllocationsError::IpOutOfRange(_) => TunnelState::ErrorIpOutOfRange,
            AllocationsError::RangeExhausted => TunnelState::ErrorIpRangeExhausted,
        },
        _ => TunnelState::ErrorCreatingTunnel,
    }
}

async fn reconcile(
    object: &Tunnel,
    context: &ReconcilerContext,
) -> Result<Action, ReconcilerError> {
    let status: TunnelStatus = prepare_status(object, context).await?;

    apply_resource_status::<Tunnel, TunnelStatus>(
        &context.client,
        status,
        object.require_name_or(ReconcilerError::MissingObjectMetadata)?,
        object.require_namespace_or(ReconcilerError::MissingObjectMetadata)?,
        &PatchParams::apply(NETWORK_MANAGER_FIELD_MANAGER),
    )
    .await
    .map_err(ReconcilerError::KubeApiError)?;

    Ok(Action::requeue(Duration::from_secs(RECONCILE_REQUEUE_SECS)))
}

async fn cleanup(object: &Tunnel, context: &ReconcilerContext) -> Result<Action, ReconcilerError> {
    let public_key = WgKey::from_base64(&object.spec.peer_public_key)
        .map_err(|_| ReconcilerError::InvalidObjectData("peer_public_key".into()))?;

    remove_address_by_key(public_key, context).await;

    Ok(Action::await_change())
}

async fn prepare_status(
    object: &Tunnel,
    context: &ReconcilerContext,
) -> Result<TunnelStatus, ReconcilerError> {
    let public_key = WgKey::from_base64(&object.spec.peer_public_key)
        .map_err(|_| ReconcilerError::InvalidObjectData("peer_public_key".into()))?;
    let mut status = match &object.status {
        Some(status) => status.clone(),
        None => TunnelStatus {
            address: None,
            state: Default::default(),
        },
    };

    if status.address.is_none() {
        status.address = match object.spec.static_ip {
            Some(ip) => get_or_insert_address(public_key, ip, context).await?,
            None => get_or_allocate_address(public_key, context).await?,
        };
    }

    Ok(status)
}

async fn get_or_allocate_address(
    key: WgKey,
    context: &ReconcilerContext,
) -> Result<Option<IpAddrPair>, ReconcilerError> {
    Ok(match context.allocations_ipv4 {
        Some(ref allocations) => Some(IpAddrPair::Ipv4 {
            ipv4: allocations
                .get_or_allocate(&key)
                .await
                .map_err(ReconcilerError::Ipv4AllocationError)?,
        }),
        _ => None, // no allocator, no addresses
    })
}

async fn get_or_insert_address(
    key: WgKey,
    ip: IpAddrPair,
    context: &ReconcilerContext,
) -> Result<Option<IpAddrPair>, ReconcilerError> {
    Ok(match context.allocations_ipv4 {
        Some(ref allocations) => match ip.try_get_ipv4() {
            Some(ipv4) => Some(IpAddrPair::Ipv4 {
                ipv4: allocations
                    .get_or_insert(&key, || ipv4)
                    .await
                    .map_err(ReconcilerError::Ipv4AllocationError)?,
            }),
            None => None,
        },
        _ => None,
    })
}

async fn remove_address_by_key(key: WgKey, context: &ReconcilerContext) {
    if let Some(ref allocator_ipv4) = context.allocations_ipv4 {
        allocator_ipv4.try_remove(&key).await;
    }
}
