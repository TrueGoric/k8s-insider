use std::{borrow::Cow, net::Ipv4Addr};

use k8s_insider_core::resources::{router::{RouterReleaseBuilderError, RouterReleaseValidationError}, ResourceGenerationError};
use thiserror::Error;

use crate::network_manager::allocations::AllocationsError;

#[derive(Debug, Error)]
pub enum ReconcilerError {
    #[error("Object is missing metadata!")]
    MissingObjectMetadata,
    #[error("'{}' resource is missing required data!", .0)]
    MissingObjectData(String),
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
    RouterReleaseResourceValidationError(RouterReleaseValidationError),
    #[error("Couldn't allocate Ipv4! Details: {}", .0)]
    Ipv4AllocationError(AllocationsError<Ipv4Addr>)

}
