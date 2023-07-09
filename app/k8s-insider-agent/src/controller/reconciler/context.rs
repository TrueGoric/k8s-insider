use k8s_insider_core::resources::controller::ControllerRelease;
use k8s_openapi::api::core::v1::Node;
use kube::{runtime::reflector::Store, Client};

pub struct ReconcilerContext {
    pub release: ControllerRelease,
    pub client: Client,
    pub nodes: Store<Node>,
}
