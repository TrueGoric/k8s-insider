use std::borrow::Cow;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReconcilerError {
    #[error("Object is missing metadata!")]
    MissingObjectMetadata,
    #[error("'{}' resource contains invalid data!", .0)]
    InvalidObjectData(Cow<'static, str>),
    #[error("Couldn't patch the resource! Reason: {}", .0)]
    KubeApiError(kube::Error),
}
