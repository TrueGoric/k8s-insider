use std::{str::from_utf8, sync::Arc, time::Duration};

use k8s_insider_core::{
    helpers::RequireMetadata,
    kubernetes::{
        operations::{apply_resource, apply_resource_status, try_get_resource},
        service::get_service_accessible_addresses,
    },
    resources::{
        crd::v1alpha1::network::{Network, NetworkState, NetworkStatus},
        meta::TryNetworkMeta,
        router::{
            secret::SERVER_PRIVATE_KEY_SECRET, RouterInfoBuilder, RouterRelease,
            RouterReleaseBuilder, RouterReleaseValidationError,
        },
    },
    wireguard::keys::{Keys, WgKey},
};
use k8s_openapi::api::core::v1::{Secret, Service};
use kube::{api::PatchParams, runtime::controller::Action, Resource};

use crate::controller::CONTROLLER_FIELD_MANAGER;

use super::{context::ReconcilerContext, error::ReconcilerError};

const SUCCESS_REQUEUE_SECS: u64 = 60 * 5;

const DEFAULT_ERROR_REQUEUE_SECS: u64 = 10;
const VALIDATION_ERROR_REQUEUE_SECS: u64 = 60 * 5;

pub async fn reconcile_network(
    object: Arc<Network>,
    context: Arc<ReconcilerContext>,
) -> Result<Action, ReconcilerError> {
    let reconcile_result = try_reconcile(&object, &context).await;

    match reconcile_result {
        Ok(_) => Ok(Action::requeue(Duration::from_secs(SUCCESS_REQUEUE_SECS))),
        Err(error) => {
            let state = get_error_state(&error);
            let status = NetworkStatus {
                state,
                ..Default::default()
            };

            apply_resource_status::<Network, NetworkStatus>(
                &context.client,
                status,
                object.require_name_or(ReconcilerError::MissingObjectMetadata)?,
                object.require_namespace_or(ReconcilerError::MissingObjectMetadata)?,
                &PatchParams::apply(CONTROLLER_FIELD_MANAGER),
            )
            .await
            .map_err(ReconcilerError::KubeApiError)?;

            Err(error)
        }
    }
}

pub fn reconcile_network_error(
    _object: Arc<Network>,
    error: &ReconcilerError,
    _context: Arc<ReconcilerContext>,
) -> Action {
    Action::requeue(match error {
        ReconcilerError::RouterReleaseResourceValidationError(_) => {
            Duration::from_secs(VALIDATION_ERROR_REQUEUE_SECS)
        }
        _ => Duration::from_secs(DEFAULT_ERROR_REQUEUE_SECS),
    })
}

async fn try_reconcile(
    object: &Network,
    context: &ReconcilerContext,
) -> Result<(), ReconcilerError> {
    let private_key = ensure_server_private_key(object, context).await?;
    let release = build_release(private_key, object, context)?
        .validated()
        .map_err(ReconcilerError::RouterReleaseResourceValidationError)?;

    apply_release(context, &release).await?;

    let service_meta = release.generate_service_metadata();
    let service = try_get_resource::<Service>(
        &context.client,
        service_meta.name.as_ref().unwrap(),
        service_meta.namespace.as_ref().unwrap(),
    )
    .await
    .map_err(ReconcilerError::KubeApiError)?;
    let nodes = context.nodes.state();
    let node_slice = nodes.iter().map(|node| node.as_ref()).collect::<Vec<_>>();

    let status = NetworkStatus {
        state: NetworkState::Deployed,
        allowed_ips: Some(release.get_allowed_fitcidrs()),
        service_domain: release.service_domain,
        dns: release.kube_dns,
        endpoints: get_service_accessible_addresses(service.as_ref(), &node_slice).await,
        server_public_key: Some(release.server_keys.get_public_key().to_base64()),
    };

    apply_resource_status::<Network, NetworkStatus>(
        &context.client,
        status,
        object.require_name_or(ReconcilerError::MissingObjectMetadata)?,
        object.require_namespace_or(ReconcilerError::MissingObjectMetadata)?,
        &PatchParams::apply(CONTROLLER_FIELD_MANAGER),
    )
    .await
    .map_err(ReconcilerError::KubeApiError)?;

    Ok(())
}

fn build_release(
    private_key: Keys,
    object: &Network,
    context: &ReconcilerContext,
) -> Result<RouterRelease, ReconcilerError> {
    let router_info = RouterInfoBuilder::default()
        .with_network_crd(object)
        .map_err(ReconcilerError::RouterReleaseBuilderResourceError)?
        .server_keys(private_key)
        .owner(object.controller_owner_ref(&()).unwrap())
        .build()
        .map_err(ReconcilerError::RouterInfoBuilderError)?;

    RouterReleaseBuilder::default()
        .with_controller(&context.release)
        .with_router_info(router_info)
        .build()
        .map_err(ReconcilerError::RouterReleaseBuilderError)
}

async fn ensure_server_private_key(
    crd: &Network,
    context: &ReconcilerContext,
) -> Result<Keys, ReconcilerError> {
    let name = crd
        .try_get_router_name()
        .ok_or(ReconcilerError::MissingObjectMetadata)?;
    let namespace = crd
        .try_get_router_namespace()
        .ok_or(ReconcilerError::MissingObjectMetadata)?;
    let secret = try_get_resource::<Secret>(&context.client, &name, &namespace)
        .await
        .map_err(ReconcilerError::KubeApiError)?;

    let private_key = match secret {
        Some(secret) => Keys::from_private_key(
            WgKey::from_base64(
                from_utf8(
                    &secret
                        .data
                        .as_ref()
                        .ok_or_else(|| ReconcilerError::MissingObjectData(name.clone()))?
                        .get(SERVER_PRIVATE_KEY_SECRET)
                        .ok_or_else(|| ReconcilerError::MissingObjectData(name.clone()))?
                        .0[..],
                )
                .map_err(|_| ReconcilerError::InvalidObjectData(name.clone().into()))?,
            )
            .map_err(|_| ReconcilerError::InvalidObjectData(name.into()))?,
        ),
        None => Keys::generate_new_pair(),
    };

    Ok(private_key)
}

async fn apply_release(
    context: &ReconcilerContext,
    release: &RouterRelease,
) -> Result<(), ReconcilerError> {
    let patch_params = PatchParams::apply(CONTROLLER_FIELD_MANAGER);

    apply_network_manager(context, release, &patch_params).await?;
    apply_router(context, release, &patch_params).await?;

    Ok(())
}

async fn apply_router(
    context: &ReconcilerContext,
    release: &RouterRelease,
    patch_params: &PatchParams,
) -> Result<(), ReconcilerError> {
    let secret = release
        .generate_secret()
        .map_err(ReconcilerError::RouterReleaseResourceGenerationError)?;
    let service_account = release.generate_router_service_account();
    let role_binding = release
        .generate_router_role_binding(&service_account)
        .map_err(ReconcilerError::RouterReleaseResourceGenerationError)?;
    let deployment = release
        .generate_router_deployment(&secret, &service_account)
        .map_err(ReconcilerError::RouterReleaseResourceGenerationError)?;
    let service = release.generate_service(&deployment);

    apply_resource(&context.client, &service_account, patch_params)
        .await
        .map_err(ReconcilerError::KubeApiError)?;
    apply_resource(&context.client, &role_binding, patch_params)
        .await
        .map_err(ReconcilerError::KubeApiError)?;
    apply_resource(&context.client, &secret, patch_params)
        .await
        .map_err(ReconcilerError::KubeApiError)?;
    apply_resource(&context.client, &deployment, patch_params)
        .await
        .map_err(ReconcilerError::KubeApiError)?;

    if let Some(service) = service {
        apply_resource(&context.client, &service, patch_params)
            .await
            .map_err(ReconcilerError::KubeApiError)?;
    }

    Ok(())
}

async fn apply_network_manager(
    context: &ReconcilerContext,
    release: &RouterRelease,
    patch_params: &PatchParams,
) -> Result<(), ReconcilerError> {
    let service_account = release.generate_network_manager_service_account();
    let role_binding = release
        .generate_network_manager_role_binding(&service_account)
        .map_err(ReconcilerError::RouterReleaseResourceGenerationError)?;
    let deployment = release
        .generate_network_manager_deployment(&service_account)
        .map_err(ReconcilerError::RouterReleaseResourceGenerationError)?;

    apply_resource(&context.client, &service_account, patch_params)
        .await
        .map_err(ReconcilerError::KubeApiError)?;
    apply_resource(&context.client, &role_binding, patch_params)
        .await
        .map_err(ReconcilerError::KubeApiError)?;
    apply_resource(&context.client, &deployment, patch_params)
        .await
        .map_err(ReconcilerError::KubeApiError)?;

    Ok(())
}

fn get_error_state(error: &ReconcilerError) -> NetworkState {
    match error {
        ReconcilerError::RouterReleaseResourceValidationError(err) => match err {
            RouterReleaseValidationError::RouterIpOutOfBounds => NetworkState::ErrorSubnetConflict,
            RouterReleaseValidationError::MissingKeys => NetworkState::ErrorCreatingService,
        },
        ReconcilerError::KubeApiError(err) => match err {
            kube::Error::Auth(_) => NetworkState::ErrorInsufficientPermissions,
            kube::Error::Api(err) => match err.code {
                403 => NetworkState::ErrorInsufficientPermissions,
                _ => NetworkState::UnknownError,
            },
            _ => NetworkState::UnknownError,
        },
        _ => NetworkState::UnknownError,
    }
}
