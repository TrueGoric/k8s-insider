use k8s_openapi::chrono::{DateTime, Utc};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    /// if set to true this tunnel won't be automatically cleaned up after
    /// being unused for a preconfigured amount of time
    pub persistent: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct TunnelStatus {
    pub state: TunnelState,
    /// last handshake 
    pub last_handshake: Option<DateTime<Utc>>,
    /// server public key
    pub server_public_key: Option<String>,
    /// dyamically assigned peer address
    pub address: Option<String>,
    /// dns address
    pub dns: Option<String>,
    /// publicly available address
    pub endpoint: Option<String>,
    /// routable ips for this tunnel
    pub allowed_ips: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub enum TunnelState {
    #[default]
    Unknown,
    Creating,
    Created,
    Closed,
    ErrorCreatingTunnel
}