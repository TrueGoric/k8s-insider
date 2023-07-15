use std::fmt::Display;

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::ip::addrpair::IpAddrPair;

#[skip_serializing_none]
#[derive(CustomResource, Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[kube(
    group = "k8s-insider.dev",
    version = "v1alpha1",
    kind = "Tunnel",
    namespaced,
    status = "TunnelStatus",
    derive = "Default"
)]
pub struct TunnelSpec {
    /// network this tunnel is attached to
    pub network: String,
    /// peer public key
    pub peer_public_key: String,
    /// tunnel's preshared key
    pub preshared_key: String,
    /// static IP of choice, the tunnel will fail to be created if it's unavailable or out of range
    /// the allocations are made on a first-come-first-served basis,
    pub static_ip: Option<IpAddrPair>,
    /// if set to true this tunnel won't be automatically cleaned up after
    /// being unused for a preconfigured amount of time
    pub persistent: bool,
}

#[skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TunnelStatus {
    pub state: TunnelState,
    /// dynamically assigned peer address
    pub address: Option<IpAddrPair>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq, JsonSchema)]
pub enum TunnelState {
    #[default]
    Created,
    Configured,
    Connected,
    Closed,
    ErrorCreatingTunnel,
    ErrorIpAlreadyInUse,
    ErrorIpOutOfRange,
    ErrorPublicKeyConflict,
    ErrorIpRangeExhausted,
}

impl Display for TunnelState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TunnelState::Created => f.write_str("tunnel created"),
            TunnelState::Configured => f.write_str("tunnel configured by the controller"),
            TunnelState::Connected => f.write_str("user connected to the tunnel"),
            TunnelState::Closed => f.write_str("tunnel is closed"),
            TunnelState::ErrorCreatingTunnel => {
                f.write_str("an error occurred while creating the tunnel")
            }
            TunnelState::ErrorIpAlreadyInUse => {
                f.write_str("tunnel requested a static IP that's already in use")
            }
            TunnelState::ErrorIpOutOfRange => {
                f.write_str("tunnel requested a static IP that's out of range")
            }
            TunnelState::ErrorPublicKeyConflict => {
                f.write_str("there's another tunnel with the same public key")
            }
            TunnelState::ErrorIpRangeExhausted => {
                f.write_str("IP range for tunnels has been exhausted")
            }
        }
    }
}
