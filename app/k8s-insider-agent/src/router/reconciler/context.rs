use k8s_insider_core::resources::{crd::v1alpha1::network::Network, router::RouterInfo};
use kube::Client;

pub struct ReconcilerContext {
    pub router_info: RouterInfo,
    pub owner: Network,
    pub client: Client,
}
