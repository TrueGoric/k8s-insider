use kube::{core::object::HasStatus, Resource};
use thiserror::Error;

pub mod controller;
pub mod annotations;
pub mod crd;
pub mod labels;
pub mod router;
pub mod templates;

#[derive(Debug, Error)]
pub enum ResourceGenerationError {
    #[error("Provided dependent resource is missing a name!")]
    DependentMissingMetadataName,
    #[error("Provided dependent resource is missing a namespace!")]
    DependentMissingMetadataNamespace
}

pub trait FromStatus<S> {
    fn from_status(status: S) -> Self;
}

impl<T: Default + HasStatus<Status = S> + ?Sized, S> FromStatus<S> for T {
    fn from_status(status: S) -> Self {
        let mut object = Self::default();

        *object.status_mut() = Some(status);

        object
    }
}