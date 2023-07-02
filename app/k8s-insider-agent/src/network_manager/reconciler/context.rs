use k8s_insider_core::resources::{controller::ControllerRelease, router::RouterRelease, crd::v1alpha1::network::Network};
use kube::Client;

use crate::network_manager::allocations::Ipv4AllocationsSync;

pub struct ReconcilerContext {
    pub controller_release: ControllerRelease,
    pub router_release: RouterRelease,
    pub owner: Network,
    pub client: Client,
    pub allocations_ipv4: Option<Ipv4AllocationsSync>,
}
