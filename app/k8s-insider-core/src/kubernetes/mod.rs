use k8s_openapi::NamespaceResourceScope;
use kube::{core::object::HasStatus, Api, Client, Resource};

pub mod operations;
pub mod service;

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

pub trait GetApi {
    fn global_api<T>(&self) -> Api<T>
    where
        T: Resource,
        <T as Resource>::DynamicType: Default;

    fn namespaced_api<T>(&self, namespace: &str) -> Api<T>
    where
        T: Resource<Scope = NamespaceResourceScope>,
        <T as Resource>::DynamicType: Default;
}

impl GetApi for Client {
    fn global_api<T>(&self) -> Api<T>
    where
        T: Resource,
        <T as Resource>::DynamicType: Default,
    {
        Api::all(self.clone())
    }

    fn namespaced_api<T>(&self, namespace: &str) -> Api<T>
    where
        T: Resource<Scope = NamespaceResourceScope>,
        <T as Resource>::DynamicType: Default,
    {
        Api::namespaced(self.clone(), namespace)
    }
}
