use std::net::IpAddr;

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::ippair::{IpAddrPair, IpNetPair};

#[skip_serializing_none]
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
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
    /// a service definition used to expose the network - if not defined the network won't be accessible
    pub network_service: Option<NetworkService>,
    /// whether to enable NAT or allow this network to interact directly with the cluster
    /// (depending on the controller implementation and cluster capabilities this might not have an effect)
    pub nat: Option<bool>,
}

#[skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum NetworkService {
    #[serde(rename_all = "camelCase")]
    ClusterIp {
        ip: Option<IpAddrPair>,
    },
    #[serde(rename_all = "camelCase")]
    NodePort {
        cluster_ip: Option<IpAddrPair>,
        predefined_ips: Option<Vec<IpAddr>>,
    },
    #[serde(rename_all = "camelCase")]
    LoadBalancer {
        cluster_ip: Option<IpAddrPair>,
    },
    #[serde(rename_all = "camelCase")]
    ExternalIp {
        cluster_ip: Option<IpAddrPair>,
        ips: Vec<IpAddr>,
    },
}

#[skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
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
    ErrorCreatingService,
    ErrorSubnetConflict,
    ErrorInsufficientPermissions,
}
