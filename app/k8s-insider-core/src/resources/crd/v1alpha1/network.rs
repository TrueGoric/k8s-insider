use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ippair::IpNetPair;

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    group = "k8s-insider.dev",
    version = "v1alpha1",
    kind = "Network",
    namespaced,
    status = "NetworkStatus"
)]
pub struct NetworkSpec {
    /// CIDR range for peers connecting to this network
    pub peer_cidr: IpNetPair,
    /// whether to enable NAT or allow this network to interact directly with the cluster
    /// (depending on the implementation and cluster capabilities this might not have an effect)
    pub nat: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct NetworkStatus {
    /// network state
    pub state: NetworkState,
    /// server public key
    pub server_public_key: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub enum NetworkState {
    #[default]
    Unknown,
    Creating,
    Created,
    ErrorSubnetConflict,
    ErrorInsufficientPermissions,
}
