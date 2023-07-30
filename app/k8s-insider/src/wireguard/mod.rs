use k8s_insider_core::resources::crd::v1alpha1::{network::NetworkState, tunnel::TunnelState};
use thiserror::Error;

pub mod connection_manager;
pub mod operations;
pub mod helpers;
pub mod peer_config;

#[derive(Debug, Error)]
pub enum WireguardError {
    #[error("No address is assigned!")]
    AddressNotAssigned,
    #[error("Network is not ready!")]
    NetworkNotReady,
    #[error("Network is in an invalid state ({})!", .0)]
    NetworkInvalidState(NetworkState),
    #[error("Network's public key is invalid!")]
    NetworkInvalidServerPublicKey,
    #[error("Network is missing a public endpoint!")]
    NetworkMissingEndpoint,
    #[error("Network is missing allowed IPs!")]
    NetworkMissingAllowedIps,
    #[error("Tunnel is not ready!")]
    TunnelNotReady,
    #[error("Tunnel is in an invalid state ({})!", .0)]
    TunnelInvalidState(TunnelState),
    #[error("Tunnel's preshared key is invalid!")]
    TunnelInvalidPresharedKey,
}
