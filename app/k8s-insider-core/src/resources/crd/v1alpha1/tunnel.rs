use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ippair::{IpAddrPair, IpNetPair};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    group = "k8s-insider.dev",
    version = "v1alpha1",
    kind = "Tunnel",
    namespaced,
    status = "TunnelStatus"
)]
pub struct TunnelSpec {
    /// peer public key
    pub peer_public_key: String,
    /// tunnel's preshared key
    pub preshared_key: String,
    /// static IP of choice, the tunnel will fail to be created if it's unavailable or out of range
    pub static_ip: Option<IpAddrPair>,
    /// if set to true this tunnel won't be automatically cleaned up after
    /// being unused for a preconfigured amount of time
    pub persistent: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct TunnelStatus {
    pub state: TunnelState,
    /// server public key
    pub server_public_key: Option<String>,
    /// dynamically assigned peer address
    pub address: Option<IpAddrPair>,
    /// dns address
    pub dns: Option<IpAddrPair>,
    /// publicly available address
    pub endpoint: Option<String>,
    /// publicly available address
    pub endpoint_port: Option<u32>,
    /// routable ip ranges for this tunnel
    pub allowed_ips: Option<Vec<IpNetPair>>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub enum TunnelState {
    #[default]
    Unknown,
    Creating,
    Created,
    Connected,
    Closed,
    ErrorCreatingTunnel,
    ErrorIpAlreadyInUse,
    ErrorIpOutOfRange,
}
