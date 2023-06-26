use kube::Resource;

use self::error::ReconcilerError;

pub mod context;
pub mod error;
pub mod network;

pub trait RequireMetadata {
    fn require_name(&self) -> Result<&str, ReconcilerError>;
    fn require_namespace(&self) -> Result<&str, ReconcilerError>;
}

impl<T: Resource> RequireMetadata for T {
    fn require_name(&self) -> Result<&str, ReconcilerError> {
        Ok(self
            .meta()
            .name
            .as_ref()
            .ok_or(ReconcilerError::MissingObjectMetadata)?
            .as_str())
    }

    fn require_namespace(&self) -> Result<&str, ReconcilerError> {
        Ok(self
            .meta()
            .namespace
            .as_ref()
            .ok_or(ReconcilerError::MissingObjectMetadata)?
            .as_str())
    }
}
