use k8s_insider_core::resources::controller::ControllerRelease;
use kube::Client;

pub struct ReconcilerContext {
    pub release: ControllerRelease,
    pub client: Client,
}
