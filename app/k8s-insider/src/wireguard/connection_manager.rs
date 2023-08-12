use std::{
    collections::HashMap,
    fs::{self},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use log::{info, warn};

use crate::{
    config::{network::NetworkIdentifier, tunnel::TunnelIdentifier},
    wireguard::operations::tunnel_connect,
};

use super::{
    operations::{tunnel_disconnect, unpatch_dns},
    peer_config::{InsiderPeerMeta, WireguardPeerConfig, WireguardPeerConfigHandle},
};

struct TunnelInfo {
    pub tunnel: TunnelIdentifier,
    pub config_path: PathBuf,
    pub meta_path: PathBuf,
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

            if config_path.extension().unwrap_or_default() != "conf" {
                continue;
            }

            let meta_path = config_path.with_extension("meta");
            let config_meta = match InsiderPeerMeta::from_file(&meta_path) {
                Ok(meta) => meta,
                Err(error) => {
                    warn!(
                        "Invalid/missing WireGuard meta file in connections, ignoring! (path: {}, error: {})",
                        meta_path.to_string_lossy(),
                        error
                    );

                    continue;
                }
            };

            let tunnel_info = TunnelInfo {
                tunnel: config_meta.tunnel,
                config_path,
                meta_path,
            };

            active_connections.insert(tunnel_info.tunnel.network.clone(), tunnel_info);
        }

        Ok(Self {
            config_directory,
            active_connections,
        })
    }

    pub fn get_peer_config<'a>(
        &'a self,
        network: &NetworkIdentifier,
    ) -> anyhow::Result<WireguardPeerConfigHandle<'a>> {
        if let Some(tunnel_info) = self.active_connections.get(network) {
            let meta = InsiderPeerMeta::from_file(&tunnel_info.meta_path)?;
            let config = WireguardPeerConfig::from_file(&tunnel_info.config_path)?;
            let handle = WireguardPeerConfigHandle {
                config,
                config_path: &tunnel_info.config_path,
                meta,
                meta_path: &tunnel_info.meta_path,
            };

            Ok(handle)
        } else {
            Err(anyhow!(
                "Couldn't find WireGuard configuration for '{}' network!",
                network.name
            ))
        }
    }

    pub fn create_connection(
        &mut self,
        meta: InsiderPeerMeta,
        peer_config: WireguardPeerConfig,
    ) -> anyhow::Result<()> {
        let peer_config_name = format!("insider{}.conf", self.active_connections.len());
        let peer_config_path = self.config_directory.join(peer_config_name);
        let meta_path = peer_config_path.with_extension("meta");

        if let Some(info) = self.active_connections.get(&meta.tunnel.network) {
            return match info.tunnel.name == meta.tunnel.name {
                true => {
                    tunnel_connect(&info.config_path)?;

                    Ok(())
                }
                false => Err(anyhow!(
                    "User is already connected to '{}' network!",
                    meta.tunnel.network.name
                )),
            };
        }

        peer_config.write(&peer_config_path).context(format!(
            "Couldn't write the configuration file to '{}'!",
            peer_config_path.to_string_lossy()
        ))?;

        meta.write(&meta_path).context(format!(
            "Couldn't write the configuration meta file to '{}'!",
            meta_path.to_string_lossy()
        ))?;

        info!(
            "WireGuard config written to '{}'...",
            peer_config_path.to_string_lossy()
        );

        tunnel_connect(&peer_config_path)?;

        let tunnel_info = TunnelInfo {
            tunnel: meta.tunnel,
            config_path: peer_config_path,
            meta_path,
        };

        let network = tunnel_info.tunnel.network.clone();

        self.active_connections.insert(network, tunnel_info);

        Ok(())
    }

    pub fn patch_dns(&mut self, network_id: &NetworkIdentifier) -> anyhow::Result<()> {
        let mut config_handle = self.get_peer_config(network_id)?;
        let cluster_domain = config_handle.meta.cluster_domain.as_ref().ok_or(anyhow!(
            "Network's cluster domain is not defined in the WireGuard peer configuration!"
        ))?;

        #[cfg(target_os = "linux")]
        {
            use crate::wireguard::operations::patch_dns_linux;

            let interface_name: &str = config_handle.path.file_stem().unwrap().to_str().unwrap();

            patch_dns_linux(interface_name, &cluster_domain)?;

            info!("Configured '{interface_name}' interface to handle DNS requests for '{cluster_domain}' domain with systemd-resolved!")
        }

        #[cfg(target_os = "windows")]
        {
            use crate::wireguard::operations::patch_dns_windows;

            let dns = config_handle
                .config
                .dns
                .as_ref()
                .ok_or(anyhow!("Network's DNS server address is not defined!"))?;

            patch_dns_windows(&dns.to_string(), cluster_domain)?;

            info!("Configured NRPT to use '{dns}' for requests to '{cluster_domain}' domain!")
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            return Err(anyhow!(
                "Can't patch DNS resolver! {} is an unsupported OS for this operation!",
                std::env::consts::OS
            ));
        }

        config_handle.meta.dns_patched = true;
        config_handle.write_all().context(format!(
            "Couldn't write the configuration and metadata files to '{}'!",
            config_handle.config_path.display()
        ))?;

        Ok(())
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
            tunnel_disconnect(&tunnel_info.config_path)?;

            if let Err(error) =
                try_unpatch_dns_resolver(&tunnel_info.meta_path, &tunnel_info.config_path)
            {
                warn!("{error}");
            }

            fs::remove_file(&tunnel_info.config_path)?;
            fs::remove_file(&tunnel_info.meta_path)?;
        }

        Ok(())
    }

    pub fn remove_connection(&mut self, network_id: &NetworkIdentifier) -> anyhow::Result<()> {
        let tunnel_info = self.active_connections.remove(network_id).ok_or(anyhow!(
            "Couldn't find WireGuard configuration for '{}' network!",
            network_id.name
        ))?;

        tunnel_disconnect(&tunnel_info.config_path)?;

        if let Err(error) =
            try_unpatch_dns_resolver(&tunnel_info.meta_path, &tunnel_info.config_path)
        {
            warn!("{error}");
        }

        fs::remove_file(&tunnel_info.config_path)?;
        fs::remove_file(&tunnel_info.meta_path)?;

        Ok(())
    }
}

fn try_unpatch_dns_resolver(meta_path: &Path, config_path: &Path) -> anyhow::Result<()> {
    let meta = InsiderPeerMeta::from_file(meta_path)
        .map_err(|error| anyhow!("Couldn't unpatch the DNS resolver! A manual cleanup might be required. (error: {error})"))?;
    let config = WireguardPeerConfig::from_file(config_path)
        .map_err(|error| anyhow!("Couldn't unpatch the DNS resolver! A manual cleanup might be required. (error: {error})"))?;

    if meta.dns_patched {
        let dns = match config.dns {
            Some(dns) => dns,
            None => return Ok(()),
        };
        let cluster_domain = match meta.cluster_domain {
            Some(domain) => domain,
            None => return Ok(()),
        };

        unpatch_dns(&dns.to_string(), &cluster_domain)
            .map_err(|error| anyhow!("Couldn't unpatch the DNS resolver! A manual cleanup might be required. (error: {error})"))?;
    }

    Ok(())
}
