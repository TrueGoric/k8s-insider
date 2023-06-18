use k8s_insider_core::resources::crd::Tunnel;
use k8s_openapi::api::core::v1::{Pod, Service};
use kube::Api;

pub struct ReconcilerContext {
    pub tunnel_api: Api<Tunnel>,
    pub pod_api: Api<Pod>,
    pub service_api: Api<Service>
}