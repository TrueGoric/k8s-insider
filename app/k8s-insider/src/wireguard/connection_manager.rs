use std::{
    collections::HashMap,
    fs::{self},
    io::{BufRead, BufReader},
    path::PathBuf,
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
                Ok(id) => id,
                Err(error) => {
                    warn!(
                        "Invalid WireGuard config in connections, ignoring! (path: {}, error: {})",
                        config_path.to_string_lossy(),
                        error
                    );

                    continue;
                }
            };

            let tunnel_info = TunnelInfo {
                path: config_path,
            };

            active_connections.insert(tunnel_id.network, tunnel_info);
        }

        Ok(Self {
            config_directory,
            active_connections,
        })
    }

    pub fn create_connection(&mut self, peer_config: WireguardPeerConfig) -> anyhow::Result<()> {
        let peer_config_name = format!("insider{}.conf", self.active_connections.len());
        let peer_config_path = self.config_directory.join(peer_config_name);

        if self
            .active_connections
            .contains_key(&peer_config.tunnel.network)
        {
            return Err(anyhow!(
                "User is already connected to '{}' network!",
                peer_config.tunnel.network.name
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

        self.active_connections
            .insert(peer_config.tunnel.network, tunnel_info);

        Ok(())
    }
    
    pub fn remove_single_connection(&mut self) -> anyhow::Result<()> {
        if self.active_connections.len() > 1 {
            return Err(anyhow!("There are multiple active connections - you must choose one!"));
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
            Err(anyhow!("Couldn't find WireGuard configuration for '{}' network!", network_id.name))
        }
    }
}
