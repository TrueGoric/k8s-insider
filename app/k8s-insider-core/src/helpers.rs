use std::{time::{SystemTime, Duration}, any::type_name};

use kube::Resource;

pub fn get_secs_since_unix_epoch() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(2137420))
        .as_secs()
}

pub fn pretty_type_name<'a, T>() -> &'a str {
    type_name::<T>().split("::").last().unwrap()
}

pub trait With<F> {
    fn with(self, op: F) -> Self;
}

impl<T, F> With<F> for T
where
    F: FnOnce(&mut Self) {
    fn with(mut self, op: F) -> Self {
        op(&mut self);

        self
    }
}

pub trait AndIf<F> {
    fn and_if(self, condition: bool, then: F) -> Self;
}

pub trait AndIfSome<F, FC> {
    fn and_if_some(self, closure: FC, then: F) -> Self;
}

impl<T, F> AndIf<F> for T
where
    F: FnOnce(Self) -> Self,
{
    fn and_if(self, condition: bool, then: F) -> Self {
        let mut obj = self;
        if condition {
            obj = then(obj);
        }

        obj
    }
}

impl<T, TC, F, FC> AndIfSome<F, FC> for T
where
    F: FnOnce(Self, TC) -> Self,
    FC: FnOnce() -> Option<TC>
{
    fn and_if_some(self, closure: FC, then: F) -> Self {
        let mut obj = self;
        if let Some(result) = closure() {
            obj = then(obj, result);
        }

        obj
    }
}

pub trait Invert<TInverted> {
    fn invert(self) -> TInverted;
}

// chaotic evil impl
impl<T, E> Invert<Result<E, T>> for Result<T, E> {
    fn invert(self) -> Result<E, T> {
        match self {
            Ok(ok) => Err(ok),
            Err(err) => Ok(err),
        }
    }
}

pub trait RequireMetadata<E> {
    fn require_name_or(&self, error: E) -> Result<&str, E>;
    fn require_namespace_or(&self, error: E) -> Result<&str, E>;
    fn require_name_or_else(&self, error: impl FnOnce() -> E) -> Result<&str, E>;
    fn require_namespace_or_else(&self, error: impl FnOnce() -> E) -> Result<&str, E>;
}

impl<T: Resource, E> RequireMetadata<E> for T {
    fn require_name_or(&self, error: E) -> Result<&str, E> {
        Ok(self
            .meta()
            .name
            .as_ref()
            .ok_or(error)?
            .as_str())
    }

    fn require_namespace_or(&self, error: E) -> Result<&str, E> {
        Ok(self
            .meta()
            .namespace
            .as_ref()
            .ok_or(error)?
            .as_str())
    }

    fn require_name_or_else(&self, error: impl FnOnce() -> E) -> Result<&str, E> {
        Ok(self
            .meta()
            .name
            .as_ref()
            .ok_or_else(error)?
            .as_str())
    }

    fn require_namespace_or_else(&self, error: impl FnOnce() -> E) -> Result<&str, E> {
        Ok(self
            .meta()
            .namespace
            .as_ref()
            .ok_or_else(error)?
            .as_str())
    }
}
