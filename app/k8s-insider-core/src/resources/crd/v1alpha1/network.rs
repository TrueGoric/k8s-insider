use std::{net::{IpAddr, SocketAddr}, fmt::Display};

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::ip::{addrpair::IpAddrPair, netpair::IpNetPair, schema::IpNetFit};

#[skip_serializing_none]
#[derive(CustomResource, Deserialize, Serialize, Default, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[kube(
    group = "k8s-insider.dev",
    version = "v1alpha1",
    kind = "Network",
    namespaced,
    status = "NetworkStatus",
    derive = "Default"
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
    ClusterIp { ip: Option<IpAddrPair> },
    #[serde(rename_all = "camelCase")]
    NodePort {
        cluster_ip: Option<IpAddrPair>,
        predefined_ips: Option<Vec<IpAddr>>,
    },
    #[serde(rename_all = "camelCase")]
    LoadBalancer { cluster_ip: Option<IpAddrPair> },
    #[serde(rename_all = "camelCase")]
    ExternalIp {
        cluster_ip: Option<IpAddrPair>,
        ips: Vec<IpAddr>,
    },
}

impl Display for NetworkService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkService::ClusterIp { .. } => f.write_str("ClusterIp"),
            NetworkService::NodePort { .. } => f.write_str("NodePort"),
            NetworkService::LoadBalancer { .. } => f.write_str("LoadBalancer"),
            NetworkService::ExternalIp { .. } => f.write_str("ExternalIp"),
        }
    }
}

#[skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatus {
    /// network state
    pub state: NetworkState,
    /// server public key
    pub server_public_key: Option<String>,
    /// dns address
    pub dns: Option<IpAddrPair>,
    /// publicly available address
    pub endpoints: Option<Vec<SocketAddr>>,
    /// routable ip ranges for this tunnel
    pub allowed_ips: Option<Vec<IpNetFit>>,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Default, PartialEq, JsonSchema)]
pub enum NetworkState {
    #[default]
    Created,
    Deployed,
    UnknownError,
    ErrorCreatingService,
    ErrorSubnetConflict,
    ErrorInsufficientPermissions,
}

impl Display for NetworkState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkState::Created => f.write_str("network was created"),
            NetworkState::Deployed => f.write_str("network is deployed"),
            NetworkState::UnknownError => f.write_str("an unknown error happenned when setting up the network"),
            NetworkState::ErrorCreatingService => f.write_str("couldn't create a Service resource for the network"),
            NetworkState::ErrorSubnetConflict => f.write_str("there was an error when assigning IPs in the network"),
            NetworkState::ErrorInsufficientPermissions => f.write_str("controller lacks sufficient permissions to set up the network"),
        }
    }
}
