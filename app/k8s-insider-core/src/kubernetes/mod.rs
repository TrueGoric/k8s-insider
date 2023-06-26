use kube::core::object::HasStatus;

pub mod operations;

pub trait FromStatus<S> {
    fn from_status(status: S) -> Self;
}

impl<T: Default + HasStatus<Status = S>, S> FromStatus<S> for T {
    fn from_status(status: S) -> Self {
        let mut object = Self::default();

        *object.status_mut() = Some(status);

        object
    }
}
