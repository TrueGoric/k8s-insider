use std::borrow::Cow;

use k8s_insider_core::resources::{router::{RouterReleaseBuilderError, RouterReleaseValidationError}, ResourceGenerationError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReconcilerError {
    #[error("Object is missing metadata!")]
    MissingObjectMetadata,
    #[error("'{}' resource is missing required data!", .0)]
    MissingObjectData(Cow<'static, str>),
    #[error("'{}' resource contains invalid data!", .0)]
    InvalidObjectData(Cow<'static, str>),
    #[error("Couldn't patch the resource! Reason: {}", .0)]
    KubeApiError(kube::Error),
    #[error("Couldn't prepare a router release! Reason: {}", .0)]
    RouterReleaseBuilderError(RouterReleaseBuilderError),
    #[error("Couldn't prepare a router release! Reason: {}", .0)]
    RouterReleaseBuilderResourceError(ResourceGenerationError),
    #[error("Couldn't generate a release resource! Reason: {}", .0)]
    RouterReleaseResourceGenerationError(ResourceGenerationError),
    #[error("The release resource is invalid! Details: {}", .0)]
    RouterReleaseResourceValidationError(RouterReleaseValidationError)
}
