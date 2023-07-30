use std::{
    collections::HashMap,
    fs::{self},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use log::{info, warn};

use crate::{
    config::{network::NetworkIdentifier, tunnel::TunnelIdentifier},
    wireguard::operations::tunnel_connect,
};

use super::{operations::tunnel_disconnect, peer_config::WireguardPeerConfig};

struct TunnelInfo {
    pub path: PathBuf,
}

pub struct ConnectionManager {
    config_directory: PathBuf,
    active_connections: HashMap<NetworkIdentifier, TunnelInfo>,
}

impl ConnectionManager {
    pub fn load(config_directory: PathBuf) -> anyhow::Result<Self> {
        fs::create_dir_all(&config_directory)?;

        let config_files = fs::read_dir(&config_directory)?;
        let mut active_connections = HashMap::new();

        for config_entry in config_files {
            let config_entry = config_entry?;
            let config_path = config_entry.path();

            if config_entry.path().extension().unwrap_or_default() != "conf" {
                continue;
            }

            let config_file = fs::File::open(&config_path)?;
            let mut reader = BufReader::new(config_file);
            let mut header = String::new();

            reader.read_line(&mut header)?;

            let tunnel_id = match TunnelIdentifier::from_wgconf_header(&header) {
                Ok(id) => match id {
                    Some(id) => id,
                    None => {
                        warn!(
                            "WireGuard config at path '{}' is missing the k8s-insider header!",
                            config_path.to_string_lossy()
                        );

                        continue;
                    }
                },
                Err(error) => {
                    warn!(
                        "Invalid WireGuard config in connections, ignoring! (path: {}, error: {})",
                        config_path.to_string_lossy(),
                        error
                    );

                    continue;
                }
            };

            let tunnel_info = TunnelInfo { path: config_path };

            active_connections.insert(tunnel_id.network, tunnel_info);
        }

        Ok(Self {
            config_directory,
            active_connections,
        })
    }

    pub fn get_peer_config_path(&self, network: &NetworkIdentifier) -> anyhow::Result<&Path> {
        match self.active_connections.get(network) {
            Some(tunnel_info) => Ok(&tunnel_info.path),
            None => Err(anyhow!(
                "Couldn't find WireGuard configuration for '{}' network!",
                network.name
            )),
        }
    }

    pub fn get_peer_config(
        &self,
        network: &NetworkIdentifier,
    ) -> anyhow::Result<WireguardPeerConfig> {
        if let Some(tunnel_info) = self.active_connections.get(network) {
            let config_file = fs::File::open(&tunnel_info.path)?;
            let mut reader = BufReader::new(config_file);

            let config = WireguardPeerConfig::from_reader(&mut reader)?;

            Ok(config)
        } else {
            Err(anyhow!(
                "Couldn't find WireGuard configuration for '{}' network!",
                network.name
            ))
        }
    }

    pub fn create_connection(&mut self, peer_config: WireguardPeerConfig) -> anyhow::Result<()> {
        let peer_config_name = format!("insider{}.conf", self.active_connections.len());
        let peer_config_path = self.config_directory.join(peer_config_name);
        let tunnel = peer_config.tunnel.as_ref().ok_or(anyhow!(
            "WireGuardPeerConfig is missing tunnel information!"
        ))?;

        if self.active_connections.contains_key(&tunnel.network) {
            return Err(anyhow!(
                "User is already connected to '{}' network!",
                tunnel.network.name
            ));
        }

        peer_config
            .write_configuration(&peer_config_path)
            .context(format!(
                "Couldn't write the configuration file to '{}'!",
                peer_config_path.to_string_lossy()
            ))?;

        info!(
            "WireGuard config written to '{}'...",
            peer_config_path.to_string_lossy()
        );

        tunnel_connect(&peer_config_path)?;

        let tunnel_info = TunnelInfo {
            path: peer_config_path,
        };

        let network = peer_config.tunnel.unwrap().network;

        self.active_connections.insert(network, tunnel_info);

        Ok(())
    }

    pub fn get_wg_tunnel_ifname(&self, network: &NetworkIdentifier) -> anyhow::Result<&str> {
        Ok(self
            .get_peer_config_path(network)?
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap())
    }

    pub fn remove_single_connection(&mut self) -> anyhow::Result<()> {
        if self.active_connections.len() > 1 {
            return Err(anyhow!(
                "There are multiple active connections - you must choose one!"
            ));
        }

        if self.active_connections.is_empty() {
            return Err(anyhow!("There are no active connections!"));
        }

        for (_, tunnel_info) in self.active_connections.drain() {
            tunnel_disconnect(&tunnel_info.path)?;
            fs::remove_file(&tunnel_info.path)?;
        }

        Ok(())
    }

    pub fn remove_connection(&mut self, network_id: &NetworkIdentifier) -> anyhow::Result<()> {
        if let Some(tunnel_info) = self.active_connections.remove(network_id) {
            tunnel_disconnect(&tunnel_info.path)?;
            fs::remove_file(&tunnel_info.path)?;

            Ok(())
        } else {
            Err(anyhow!(
                "Couldn't find WireGuard configuration for '{}' network!",
                network_id.name
            ))
        }
    }
}
