use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReconcilerError {
    #[error("Couldn't patch the Tunnel! Reason: {}", .0)]
    PatchError(kube::Error)
}