use k8s_insider_core::resources::controller::ControllerRelease;
use kube::{Api, Client, Resource};

pub struct ReconcilerContext {
    pub release: ControllerRelease,
    pub client: Client,
}

impl ReconcilerContext {
    pub fn global_api<T>(&self) -> Api<T>
    where
        T: Resource,
        <T as Resource>::DynamicType: Default,
    {
        Api::all(self.client.clone())
    }
}
