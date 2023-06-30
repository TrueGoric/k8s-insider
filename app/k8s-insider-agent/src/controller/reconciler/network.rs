use std::{sync::Arc, time::Duration};

use k8s_insider_core::{
    kubernetes::operations::{apply_resource, apply_resource_status, try_get_resource},
    resources::{
        crd::v1alpha1::network::{Network, NetworkState, NetworkStatus},
        router::{
            secret::SERVER_PRIVATE_KEY_SECRET, RouterRelease, RouterReleaseBuilder,
            RouterReleaseValidationError,
        },
    },
    wireguard::keys::Keys,
};
use k8s_openapi::api::core::v1::Secret;
use kube::{api::PatchParams, runtime::controller::Action, Resource};

use crate::controller::CONTROLLER_FIELD_MANAGER;

use super::{context::ReconcilerContext, error::ReconcilerError, RequireMetadata};

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

            let _ = apply_status(
                &context,
                object.require_name()?,
                object.require_namespace()?,
                state,
                None,
            )
            .await;

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
    let mut release = build_release(object, context)?;

    ensure_server_private_key(&mut release, context).await?;

    let release = release
        .validated()
        .map_err(ReconcilerError::RouterReleaseResourceValidationError)?;

    apply_release(context, &release).await?;
    apply_status(
        context,
        object.require_name()?,
        object.require_namespace()?,
        NetworkState::Deployed,
        Some(release.server_keys.unwrap().get_public_key().to_base64()),
    )
    .await?;

    Ok(())
}

fn build_release(
    object: &Network,
    context: &ReconcilerContext,
) -> Result<RouterRelease, ReconcilerError> {
    RouterReleaseBuilder::default()
        .without_server_keys()
        .with_controller(&context.release)
        .with_network_crd(object)
        .map_err(ReconcilerError::RouterReleaseBuilderResourceError)?
        .owner(object.controller_owner_ref(&()).unwrap())
        .build()
        .map_err(ReconcilerError::RouterReleaseBuilderError)
}

async fn ensure_server_private_key(
    release: &mut RouterRelease,
    context: &ReconcilerContext,
) -> Result<(), ReconcilerError> {
    let name = release.get_name();
    let namespace = release.get_namespace();
    let secret = try_get_resource::<Secret>(&context.client, &name, &namespace)
        .await
        .map_err(ReconcilerError::KubeApiError)?;

    let private_key = match secret {
        Some(secret) => Keys::from_private_key(
            secret
                .data
                .as_ref()
                .ok_or_else(|| ReconcilerError::MissingObjectData(name.clone().into()))?
                .get(SERVER_PRIVATE_KEY_SECRET)
                .ok_or_else(|| ReconcilerError::MissingObjectData(name.clone().into()))?
                .0
                .as_slice()
                .try_into()
                .map_err(|_| ReconcilerError::InvalidObjectData(name.into()))?,
        ),
        None => Keys::generate_new_pair(),
    };

    release.server_keys = Some(private_key);

    Ok(())
}

async fn apply_release(
    context: &ReconcilerContext,
    release: &RouterRelease,
) -> Result<(), ReconcilerError> {
    let secret = release
        .generate_secret()
        .map_err(ReconcilerError::RouterReleaseResourceGenerationError)?;
    let service_account = release.generate_router_service_account();
    let role_binding = release
        .generate_router_role_binding(&service_account)
        .map_err(ReconcilerError::RouterReleaseResourceGenerationError)?;
    let deployment = release
        .generate_deployment(&secret, &service_account)
        .map_err(ReconcilerError::RouterReleaseResourceGenerationError)?;
    let service = release.generate_service(&deployment);
    let patch_params = PatchParams::apply(CONTROLLER_FIELD_MANAGER);

    apply_resource(&context.client, &service_account, &patch_params)
        .await
        .map_err(ReconcilerError::KubeApiError)?;
    apply_resource(&context.client, &role_binding, &patch_params)
        .await
        .map_err(ReconcilerError::KubeApiError)?;
    apply_resource(&context.client, &secret, &patch_params)
        .await
        .map_err(ReconcilerError::KubeApiError)?;
    apply_resource(&context.client, &deployment, &patch_params)
        .await
        .map_err(ReconcilerError::KubeApiError)?;

    if let Some(service) = service {
        apply_resource(&context.client, &service, &patch_params)
            .await
            .map_err(ReconcilerError::KubeApiError)?;
    }

    Ok(())
}

async fn apply_status(
    context: &ReconcilerContext,
    name: &str,
    namespace: &str,
    state: NetworkState,
    server_public_key: Option<String>,
) -> Result<(), ReconcilerError> {
    let status = NetworkStatus {
        state,
        server_public_key,
    };

    apply_resource_status::<Network, NetworkStatus>(
        &context.client,
        status,
        name,
        namespace,
        &PatchParams::apply(CONTROLLER_FIELD_MANAGER),
    )
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
