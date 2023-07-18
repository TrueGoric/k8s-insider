use std::{io, net::SocketAddr, path::Path, fs};

use ipnet::IpNet;
use k8s_insider_core::{
    ip::addrpair::IpAddrPair,
    resources::crd::v1alpha1::{
        network::{Network, NetworkState},
        tunnel::{Tunnel, TunnelState},
    },
    wireguard::keys::WgKey,
};
use thiserror::Error;

pub mod connection;
pub mod linux;

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

pub struct WireguardPeerConfig {
    pub address: IpAddrPair,
    pub dns: Option<IpAddrPair>,

    pub peer_private_key: WgKey,
    pub server_public_key: WgKey,
    pub preshared_key: WgKey,

    pub server_endpoint: SocketAddr,
    pub allowed_ips: Vec<IpNet>,
}

impl WireguardPeerConfig {
    pub fn from_crd(
        peer_private_key: WgKey,
        network: &Network,
        tunnel: &Tunnel,
    ) -> Result<Self, WireguardError> {
        if let Some(ref network_status) = network.status {
            if network_status.state != NetworkState::Deployed {
                return Err(WireguardError::NetworkInvalidState(network_status.state));
            }

            if let Some(ref tunnel_status) = tunnel.status {
                if tunnel_status.state != TunnelState::Configured
                    && tunnel_status.state != TunnelState::Connected
                {
                    return Err(WireguardError::TunnelInvalidState(tunnel_status.state));
                }

                let address = tunnel_status
                    .address
                    .ok_or(WireguardError::AddressNotAssigned)?;
                let dns = network_status.dns;
                let server_public_key = network_status
                    .server_public_key
                    .as_deref()
                    .and_then(|k| WgKey::from_base64(k).ok())
                    .ok_or(WireguardError::NetworkInvalidServerPublicKey)?;
                let preshared_key = WgKey::from_base64(&tunnel.spec.preshared_key)
                    .map_err(|_| WireguardError::TunnelInvalidPresharedKey)?;
                let server_endpoint = network_status
                    .endpoints
                    .as_deref()
                    .and_then(|e| e.iter().next())
                    .ok_or(WireguardError::NetworkMissingEndpoint)?
                    .to_owned();
                let allowed_ips = network_status
                    .allowed_ips
                    .as_deref()
                    .map(|v| v.iter().map(|ip| ip.into()).collect())
                    .ok_or(WireguardError::NetworkMissingAllowedIps)?;

                Ok(WireguardPeerConfig {
                    address,
                    dns,
                    peer_private_key,
                    server_public_key,
                    preshared_key,
                    server_endpoint,
                    allowed_ips,
                })
            } else {
                Err(WireguardError::TunnelNotReady)
            }
        } else {
            Err(WireguardError::NetworkNotReady)
        }
    }

    pub fn generate_configuration_file(&self) -> String {
        let address = self.address;
        let private_key = self.peer_private_key.to_base64();
        let dns = self.dns.map(|i| format!("DNS = {i}")).unwrap_or_default();
        let public_key = self.server_public_key.to_base64();
        let preshared_key = self.preshared_key.to_base64();
        let endpoint = self.server_endpoint;
        let allowed_ips = self
            .allowed_ips
            .iter()
            .map(|ip| ip.to_string())
            .collect::<Vec<_>>()
            .join(",");

        format!(
            "[Interface]
Address = {address}
PrivateKey = {private_key}
{dns}

[Peer]
PublicKey = {public_key}
PresharedKey = {preshared_key}
Endpoint = {endpoint}
AllowedIPs = {allowed_ips}"
        )
    }

    pub fn write_configuration(&self, path: &Path) -> Result<(), io::Error> {
        let config = self.generate_configuration_file();

        fs::write(path, config)?;

        Ok(())
    }
}