use std::{borrow::Cow, net::Ipv4Addr};

use thiserror::Error;

use crate::network_manager::allocations::AllocationsError;

#[derive(Debug, Error)]
pub enum ReconcilerError {
    #[error("Object is missing metadata!")]
    MissingObjectMetadata,
    #[error("'{}' resource contains invalid data!", .0)]
    InvalidObjectData(Cow<'static, str>),
    #[error("Couldn't patch the resource! Reason: {}", .0)]
    KubeApiError(kube::Error),
    #[error("Couldn't allocate Ipv4! Details: {}", .0)]
    Ipv4AllocationError(AllocationsError<Ipv4Addr>),
}
