use thiserror::Error;

pub mod annotations;
pub mod controller;
pub mod crd;
pub mod labels;
pub mod router;
pub mod templates;

#[derive(Debug, Error)]
pub enum ResourceGenerationError {
    #[error("Provided dependent resource is missing a name!")]
    DependentMissingMetadataName,
    #[error("Provided dependent resource is missing a namespace!")]
    DependentMissingMetadataNamespace,
}
