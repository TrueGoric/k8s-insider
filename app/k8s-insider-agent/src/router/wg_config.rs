use std::{collections::HashMap, time::Duration};

use k8s_insider_core::{
    ip::addrpair::DualStackTryGet,
    resources::crd::v1alpha1::tunnel::{Tunnel, TunnelState, TunnelStatus},
    wireguard::keys::WgKey,
};
use kube::runtime::reflector::Store;
use log::{error, info, warn};
use tokio::sync::watch::Receiver;

use wireguard_control::{Backend, Device, DeviceUpdate, PeerConfigBuilder, PeerInfo};

use crate::wireguard::ConvertKey;

use super::reconciler::context::ReconcilerContext;

const INTERFACE_NAME: &str = "wg0";
const REFRESH_INTERVAL_SECS: u64 = 2;
const PERSISTENT_KEEPALIVE_INTERVAL_SECS: u16 = 2 * 60;

pub enum LoopCommand {
    Continue,
    Break,
}

pub struct ConfigurationSynchronizer {
    _context: ReconcilerContext,
    refresh_signal: Receiver<()>,
    tunnels: Store<Tunnel>,
}

impl ConfigurationSynchronizer {
    pub fn new(
        _context: ReconcilerContext,
        store: Store<Tunnel>,
        refresh_signal: Receiver<()>,
    ) -> Self {
        Self {
            _context,
            refresh_signal,
            tunnels: store,
        }
    }

    pub async fn start(&mut self) {
        info!("Starting WireGuard configuration synchronization...");

        loop {
            tokio::time::sleep(Duration::from_secs(REFRESH_INTERVAL_SECS)).await;

            if let LoopCommand::Break = self.synchronize().await {
                break;
            }
        }

        info!("Exiting WireGuard configuration synchronization...");
    }

    async fn synchronize(&mut self) -> LoopCommand {
        let signal_received = match self.refresh_signal.has_changed() {
            Ok(has_changed) => match self.refresh_signal.changed().await {
                Ok(_) => has_changed,
                Err(_) => return LoopCommand::Break,
            },
            Err(_) => return LoopCommand::Break,
        };

        if signal_received {
            info!("Synchronizing tunnels...");

            self.refresh_interface_config();

            info!("Tunnels synchronized!");
        }

        LoopCommand::Continue
    }

    fn refresh_interface_config(&self) {
        let mut builder = DeviceUpdate::new();
        let interface_name = INTERFACE_NAME.parse().unwrap();
        let current_status = match Device::get(&interface_name, Backend::Kernel) {
            Ok(device) => device,
            Err(err) => {
                error!("Unable to retrieve interface info! {err:#?}");
                return;
            }
        };

        let remote_peers = self.tunnels.state();
        let mut local_peers = current_status
            .peers
            .into_iter()
            .map(|i| (i.config.public_key.to_owned().convert(), i))
            .collect::<HashMap<WgKey, PeerInfo>>();

        for tunnel in remote_peers {
            let extracted_info = match Self::extract_tunnel_info(&tunnel) {
                Ok(info) => info,
                Err(_) => continue,
            };
            let (name, key, preshared_key, status) = extracted_info;

            if let Some(local_peer_info) = local_peers.remove(&key) {
                let changed_preshared_key = get_new_if_changed(
                    local_peer_info.config.preshared_key,
                    Some(preshared_key.convert()),
                );
                let changed_ip = get_new_if_changed(
                    local_peer_info
                        .config
                        .allowed_ips
                        .first()
                        .and_then(|ip| ip.address.try_get_ipv4()),
                    status.address.and_then(|address| address.try_get_ipv4()),
                ); //for now IPv4 only

                if changed_ip.is_some() || changed_preshared_key.is_some() {
                    info!("Updating peer {key} (tunnel: {name})...");

                    let mut peer = PeerConfigBuilder::new(&key.convert());

                    if let Some(ip) = changed_ip {
                        peer = peer.add_allowed_ip(ip.into(), 32).replace_allowed_ips();
                    }

                    if let Some(preshared_key) = changed_preshared_key {
                        peer = peer.set_preshared_key(preshared_key);
                    }

                    builder = builder.add_peer(peer);
                }
            } else {
                info!("Adding new peer {key} (tunnel {name})...");

                let mut peer = PeerConfigBuilder::new(&key.convert())
                    .set_persistent_keepalive_interval(PERSISTENT_KEEPALIVE_INTERVAL_SECS)
                    .set_preshared_key(preshared_key.convert());

                if let Some(ipv4) = status.address.and_then(|address| address.try_get_ipv4()) {
                    peer = peer.add_allowed_ip(ipv4.into(), 32);
                }

                builder = builder.add_peer(peer);
            }
        }

        for (leftover_key, _) in local_peers {
            info!("Removing peer {leftover_key}...");

            builder = builder.remove_peer_by_key(&leftover_key.convert());
        }

        if let Err(error) = builder.apply(&interface_name, Backend::Kernel) {
            error!("Coudldn't update {interface_name}! {error:#?}");
        }
    }

    fn extract_tunnel_info(tunnel: &Tunnel) -> Result<(&String, WgKey, WgKey, &TunnelStatus), ()> {
        let name = match tunnel.metadata.name {
            Some(ref name) => name,
            None => {
                warn!("Retrieved a tunnel without a name! Configuration for this peer won't be generated!");
                return Err(());
            }
        };
        let key = match WgKey::from_base64(&tunnel.spec.peer_public_key) {
            Ok(key) => key,
            Err(_) => {
                warn!("Invalid public key detected in the tunnel spec ({name})! Configuration for this peer won't be generated!");
                return Err(());
            }
        };
        let preshared_key = match WgKey::from_base64(&tunnel.spec.preshared_key) {
            Ok(key) => key,
            Err(_) => {
                warn!("Invalid preshared key detected in the tunnel spec ({name})! Configuration for this peer won't be generated!");
                return Err(());
            }
        };
        let status = match &tunnel.status {
            Some(status) => match status.state {
                TunnelState::Configured => status,
                TunnelState::Connected => status,
                _ => return Err(()),
            },
            None => return Err(()),
        };

        Ok((name, key, preshared_key, status))
    }
}

fn get_new_if_changed<T: PartialEq>(old: Option<T>, new: Option<T>) -> Option<T> {
    if let Some(old) = old {
        if let Some(new) = new {
            if new == old {
                return None;
            }

            return Some(new);
        }

        return None;
    }

    new
}
