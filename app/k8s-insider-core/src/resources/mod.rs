use std::borrow::Cow;

use thiserror::Error;

pub mod annotations;
pub mod controller;
pub mod crd;
pub mod labels;
pub mod meta;
pub mod router;

#[derive(Debug, Error)]
pub enum ResourceGenerationError {
    #[error("Resource contains invalid data ({})!", .0)]
    InvalidData(Cow<'static, str>),
    #[error("Resource is missing required data ({})!", .0)]
    MissingData(Cow<'static, str>),
    #[error("Provided dependent resource is missing a name!")]
    DependentMissingMetadataName,
    #[error("Provided dependent resource is missing a namespace!")]
    DependentMissingMetadataNamespace,
    #[error("Provided dependent resource is missing a namespace!")]
    DependentMissingData(Cow<'static, str>),
    #[error("Provided dependent resource contains invalid value!")]
    DependentInvalidData(Cow<'static, str>),
}
