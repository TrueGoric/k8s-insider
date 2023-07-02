use std::{sync::Arc, time::Duration};

use k8s_insider_core::{
    helpers::RequireMetadata,
    ip::addrpair::IpAddrPair,
    kubernetes::{
        operations::{apply_resource_status, try_get_resource},
        GetApi,
    },
    resources::crd::v1alpha1::{
        connection::Connection,
        tunnel::{Tunnel, TunnelState, TunnelStatus},
    },
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

use crate::{error::ReconcilerError, network_manager::NETWORK_MANAGER_FIELD_MANAGER};

use super::context::ReconcilerContext;

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

    finalizer(&tunnel_api, &finalizer_name, object, |event| async move {
        match event {
            FinalizerEvent::Apply(tunnel) => reconcile(tunnel, context).await,
            FinalizerEvent::Cleanup(_) => todo!(),
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
        FinalizerError::ApplyFailed(ReconcilerError::Ipv4AllocationError(_)) => {
            Duration::from_secs(USER_ERROR_REQUEUE_SECS)
        }
        _ => Duration::from_secs(ERROR_REQUEUE_SECS),
    })
}

async fn reconcile(
    object: Arc<Tunnel>,
    context: Arc<ReconcilerContext>,
) -> Result<Action, ReconcilerError> {
    let connection = try_get_connection(&object, &context).await?;
    let status: TunnelStatus = try_prepare_status(&object, connection, &context).await?;

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

async fn try_get_connection(
    object: &Tunnel,
    context: &ReconcilerContext,
) -> Result<Option<Connection>, ReconcilerError> {
    try_get_resource(
        &context.client,
        object.require_name_or(ReconcilerError::MissingObjectMetadata)?,
        object.require_namespace_or(ReconcilerError::MissingObjectMetadata)?,
    )
    .await
    .map_err(ReconcilerError::KubeApiError)
}

async fn try_prepare_status(
    object: &Tunnel,
    connection: Option<Connection>,
    context: &ReconcilerContext,
) -> Result<TunnelStatus, ReconcilerError> {
    let public_key = WgKey::from_base64(&object.spec.peer_public_key)
        .map_err(|_| ReconcilerError::InvalidObjectData("peer_public_key".into()))?;
    let mut status = match &object.status {
        Some(status) => status.clone(),
        None => TunnelStatus {
            address: None,
            allowed_ips: Some(context.router_release.get_allowed_fitcidrs()),
            dns: context.router_release.kube_dns,
            endpoint: None,
            endpoint_port: None,
            server_public_key: Some(
                context
                    .router_release
                    .server_keys
                    .get_public_key()
                    .to_base64(),
            ),
            state: Default::default(),
        },
    };

    if status.address.is_none() {
        status.address = match object.spec.static_ip {
            Some(ip) => try_get_or_insert_address(public_key, ip, context).await?,
            None => try_get_or_allocate_address(public_key, context).await?,
        };
    }

    status.state = match &connection {
        Some(_) => TunnelState::Connected,
        None => TunnelState::Configured,
    };

    Ok(status)
}

async fn try_get_or_allocate_address(
    key: WgKey,
    context: &ReconcilerContext,
) -> Result<Option<IpAddrPair>, ReconcilerError> {
    Ok(match context.allocations_ipv4.as_ref() {
        Some(allocations) => Some(IpAddrPair::Ipv4 {
            ipv4: allocations
                .get_or_allocate(&key)
                .await
                .map_err(ReconcilerError::Ipv4AllocationError)?,
        }),
        _ => None, // no allocator, no addresses
    })
}

async fn try_get_or_insert_address(
    key: WgKey,
    ip: IpAddrPair,
    context: &ReconcilerContext,
) -> Result<Option<IpAddrPair>, ReconcilerError> {
    Ok(match context.allocations_ipv4.as_ref() {
        Some(allocations) => match ip.try_get_ipv4() {
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