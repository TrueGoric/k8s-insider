use std::{
    borrow::Cow,
    fs,
    io::{self, BufRead, Seek},
    net::{AddrParseError, SocketAddr},
    path::Path,
};

use ini::Ini;
use ipnet::IpNet;
use k8s_insider_core::{
    ip::{addrpair::IpAddrPair, IpPairError},
    resources::crd::v1alpha1::{
        network::{Network, NetworkState},
        tunnel::{Tunnel, TunnelState},
    },
    wireguard::keys::{InvalidWgKey, WgKey},
};
use thiserror::Error;

use crate::config::tunnel::TunnelIdentifier;

use super::WireguardError;

#[derive(Debug, Error)]
pub enum WireguardParseError {
    #[error("Invalid config file! {}", .0)]
    IniError(ini::Error),
    #[error("Config file is missing {} section!", .0)]
    MissingSection(Cow<'static, str>),
    #[error("Config file is missing {} value!", .0)]
    MissingValue(Cow<'static, str>),
    #[error("This WireGuard config has an invalid header!")]
    HeaderParseError,
    #[error("Value {} contains invalid IP pair value!", .1)]
    IpPairParseError(IpPairError, Cow<'static, str>),
    #[error("Value {} contains an invalid base64 WireGuard key!", .1)]
    WgKeyParseError(InvalidWgKey, Cow<'static, str>),
    #[error("Value {} contains an invalid endpoint!", .1)]
    SocketAddrParseError(AddrParseError, Cow<'static, str>),
    #[error("Value {} contains an invalid allowed IP!", .0)]
    IpNetParseError(Cow<'static, str>),
}

pub struct WireguardPeerConfig {
    pub tunnel: Option<TunnelIdentifier>,

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
        tunnel_id: Option<&TunnelIdentifier>,
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
                    tunnel: tunnel_id.map(|id| id.to_owned()),
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
        let header = self
            .tunnel
            .as_ref()
            .map(|t| t.generate_wgconf_header())
            .unwrap_or_default();
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
            "{header}
[Interface]
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

    pub fn from_reader<R: BufRead + Seek>(reader: &mut R) -> Result<Self, WireguardParseError> {
        let mut header_buffer = String::new();
        reader.read_line(&mut header_buffer).unwrap();

        let tunnel = match TunnelIdentifier::from_wgconf_header(&header_buffer) {
            Ok(id) => id,
            Err(_) => return Err(WireguardParseError::HeaderParseError),
        };

        reader.rewind().unwrap();

        let ini = Ini::read_from(reader).map_err(WireguardParseError::IniError)?;
        let interface_section = ini
            .section(Some("Interface"))
            .ok_or(WireguardParseError::MissingSection("Interface".into()))?;

        let address = interface_section
            .get("Address")
            .ok_or(WireguardParseError::MissingValue(
                "Interface:Address".into(),
            ))?
            .parse()
            .map_err(|err| {
                WireguardParseError::IpPairParseError(err, "Interface:Address".into())
            })?;
        let peer_private_key = WgKey::from_base64(interface_section.get("PrivateKey").ok_or(
            WireguardParseError::MissingValue("Interface:PrivateKey".into()),
        )?)
        .map_err(|err| WireguardParseError::WgKeyParseError(err, "Interface:PrivateKey".into()))?;

        let dns = match interface_section.get("DNS") {
            Some(dns) => Some(dns.parse().map_err(|err| {
                WireguardParseError::IpPairParseError(err, "Interface:DNS".into())
            })?),
            None => None,
        };

        let peer_section = ini
            .section(Some("Peer"))
            .ok_or(WireguardParseError::MissingSection("Peer".into()))?;

        let server_public_key = WgKey::from_base64(
            peer_section
                .get("PublicKey")
                .ok_or(WireguardParseError::MissingValue("Peer:PublicKey".into()))?,
        )
        .map_err(|err| WireguardParseError::WgKeyParseError(err, "Peer:PublicKey".into()))?;

        let preshared_key = WgKey::from_base64(peer_section.get("PresharedKey").ok_or(
            WireguardParseError::MissingValue("Peer:PresharedKey".into()),
        )?)
        .map_err(|err| WireguardParseError::WgKeyParseError(err, "Peer:PresharedKey".into()))?;

        let server_endpoint = peer_section
            .get("Endpoint")
            .ok_or(WireguardParseError::MissingValue("Peer:Endpoint".into()))?
            .parse()
            .map_err(|err| {
                WireguardParseError::SocketAddrParseError(err, "Peer:Endpoint".into())
            })?;

        let mut invalid_allowed_ip = false;
        let allowed_ips = peer_section
            .get("AllowedIPs")
            .ok_or(WireguardParseError::MissingValue("Peer:AllowedIPs".into()))?
            .split(',')
            .map_while(|ip| match ip.parse() {
                Ok(ip) => Some(ip),
                Err(_) => {
                    invalid_allowed_ip = true;
                    None
                }
            })
            .collect();

        if invalid_allowed_ip {
            return Err(WireguardParseError::IpNetParseError(
                "Peer:AllowedIPs".into(),
            ));
        }

        Ok(Self {
            tunnel,
            address,
            dns,
            peer_private_key,
            server_public_key,
            preshared_key,
            server_endpoint,
            allowed_ips,
        })
    }
}
