use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr},
    ops::{Deref, DerefMut},
    process::exit, fmt::Display,
};

use ipnet::{Ipv4Net, Ipv6Net};
use k8s_insider_core::{
    helpers::Invert,
    ip::{Contains, Range},
    kubernetes::operations::{list_resources, try_remove_resource},
    resources::{crd::v1alpha1::tunnel::Tunnel, router::RouterRelease},
    wireguard::keys::WgKey,
};
use kube::{
    api::{DeleteParams, ListParams},
    Client,
};
use log::{error, info, warn};
use thiserror::Error;
use tokio::sync::RwLock;

pub type Ipv4Allocations = Allocations<Ipv4Addr, Ipv4Net>;

#[derive(Debug)]
pub struct Allocations<IP, IPNet> {
    range: IPNet,
    allocations: HashMap<WgKey, IP>,
    /// a _sorted_ list of allocated IP addresses
    reserved: Vec<IP>,
}

impl<IP, IPNet> Allocations<IP, IPNet>
where
    IP: Copy + Clone + Ord + Display,
    IPNet: Contains<IP> + Range<IP>,
{
    pub fn new(range: IPNet) -> Self {
        Self {
            range,
            allocations: HashMap::new(),
            reserved: Vec::new(),
        }
    }

    pub fn try_insert(&mut self, key: WgKey, ip: IP) -> Result<IP, AllocationsError<IP>> {
        if self.allocations.contains_key(&key) {
            return Err(AllocationsError::WgKeyConflict(key));
        }

        if !self.is_in_range(&ip) {
            return Err(AllocationsError::IpOutOfRange(ip));
        }

        if self.allocations.insert(key, ip).is_some() {
            panic!("Allocation insert returned a value that wasn't supposed to be there!");
        }

        self.reserved.insert(
            self.reserved
                .binary_search(&ip)
                .invert()
                .map_err(|_| AllocationsError::IpConflict(ip))?,
            ip,
        );

        info!("Allocated {ip} address!");

        Ok(ip)
    }

    pub fn try_allocate(&mut self, key: WgKey) -> Result<IP, AllocationsError<IP>> {
        if self.allocations.contains_key(&key) {
            return Err(AllocationsError::WgKeyConflict(key));
        }

        let (ip, index) =
            self.try_get_next_allocatable_ip_internal()
                .ok_or(AllocationsError::RangeExhausted)?;

        if self.allocations.insert(key, ip).is_some() {
            panic!("Allocation insert returned a value that wasn't supposed to be there!");
        }

        self.reserved.insert(index, ip);

        info!("Allocated {ip} address!");

        Ok(ip)
    }

    pub fn try_get_next_allocatable_ip(&self) -> Option<IP> {
        let mut reserved_iter = self.reserved.iter();

        // might get slow on restart for large networks, if that happens
        // this naive approach could get swapped for something more applicable
        // after profiling
        for avaliable_ip in self.range.range() {
            if let Some(reserved_ip) = reserved_iter.next() {
                if &avaliable_ip == reserved_ip {
                    continue;
                }
            }

            return Some(avaliable_ip);
        }

        None
    }

    pub fn is_in_range(&self, ip: &IP) -> bool {
        self.range.contains(ip)
    }

    fn try_get_next_allocatable_ip_internal(&self) -> Option<(IP, usize)> {
        let ip = self.try_get_next_allocatable_ip()?;
        let position = self.reserved.binary_search(&ip).unwrap_err();

        Some((ip, position))
    }
}


#[derive(Debug, Error)]
pub enum AllocationsError<IP: Display> {
    #[error("Public key '{}' is already present in the allocations table!", .0)]
    WgKeyConflict(WgKey),
    #[error("Address {} is already allocated!", .0)]
    IpConflict(IP),
    #[error("The IP range for this network was exhausted!")]
    RangeExhausted,
    #[error("Address {} is out of range!", .0)]
    IpOutOfRange(IP),
}

pub type Ipv4AllocationsSync = AllocationsSync<Ipv4Addr, Ipv4Net>;
pub type Ipv6AllocationsSync = AllocationsSync<Ipv6Addr, Ipv6Net>;

pub struct AllocationsSync<IP, IPNet>(RwLock<Allocations<IP, IPNet>>);

impl<IP, IPNet> AllocationsSync<IP, IPNet>
where
    IP: Copy + Clone + Ord + Display,
    IPNet: Contains<IP> + Range<IP>,
{
    pub async fn get_or_allocate(&self, key: &WgKey) -> Result<IP, AllocationsError<IP>> {
        {
            let read_guard = self.read().await;

            if let Some(ip) = read_guard.allocations.get(key) {
                return Ok(*ip);
            }
        }

        let mut guard = self.write().await;
        
        if let Some(ip) = guard.allocations.get(key) {
            return Ok(*ip);
        }

        guard.try_allocate(key.to_owned())
    }

    pub async fn get_or_insert<F: FnOnce() -> IP>(&self, key: &WgKey, ip_getter: F) -> Result<IP, AllocationsError<IP>>
    {
        {
            let read_guard = self.read().await;

            if let Some(ip) = read_guard.allocations.get(key) {
                return Ok(*ip);
            }
        }

        let mut guard = self.write().await;
        
        if let Some(ip) = guard.allocations.get(key) {
            return Ok(*ip);
        }

        guard.try_insert(key.to_owned(), ip_getter())
    }
}

impl<IP, IPNet> Deref for AllocationsSync<IP, IPNet> {
    type Target = RwLock<Allocations<IP, IPNet>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<IP, IPNet> DerefMut for AllocationsSync<IP, IPNet> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<IP, IPNet> From<Allocations<IP, IPNet>> for AllocationsSync<IP, IPNet> {
    fn from(value: Allocations<IP, IPNet>) -> Self {
        Self(RwLock::new(value))
    }
}

pub async fn sync_allocations(
    client: &Client,
    router_release: &RouterRelease,
) -> Result<(Option<Ipv4AllocationsSync>, Option<Ipv6AllocationsSync>), kube::Error> {
    info!("Synchronizing address allocations...");

    let tunnels = list_resources(client, &router_release.namespace, &ListParams::default()).await?;
    
    let (allocations_ipv4, conflicting_tunnels) = match init_ipv4_allocations(router_release, tunnels.iter()) {
        Some(result) => result,
        None => {
            error!("No IPv4 peer CIDR or router IP defined by the network! Can't continue!");
            exit(100);
        }
    };

    let delete_params = DeleteParams::background();
    for tunnel in conflicting_tunnels {
        let tunnel_name = match &tunnel.metadata.name {
            Some(name) => name,
            None => continue,
        };
        let tunnel_namespace = match &tunnel.metadata.namespace {
            Some(namespace) => namespace,
            None => continue,
        };

        warn!("Removing '{tunnel_name}' due to conflicting IP address ({:?})! Someone was naughty!", tunnel.status.as_ref().and_then(|s| s.address));

        try_remove_resource::<Tunnel>(client, tunnel_name, tunnel_namespace, &delete_params)
            .await?;
    }
    info!("Address allocations synchronized!");

    Ok((Some(allocations_ipv4.into()), None))
}

pub fn init_ipv4_allocations<'a>(
    release: &RouterRelease,
    existing_tunnels: impl IntoIterator<Item = &'a Tunnel>,
) -> Option<(Ipv4Allocations, Vec<&'a Tunnel>)> {
    let mut allocations = Ipv4Allocations::new(release.peer_cidr.try_get_ipv4()?);
    let mut troublemakers = Vec::new();

    allocations.try_insert(release.server_keys.get_public_key().to_owned(), release.router_ip.try_get_ipv4()?).unwrap();

    // fill only the already allocated ips, the rest
    // will be handled by the controller
    for tunnel in existing_tunnels {
        let established_address = tunnel
            .status
            .as_ref()
            .and_then(|status| status.address)
            .and_then(|address| address.try_get_ipv4());

        if let Some(address) = established_address {
            let key = WgKey::from_base64(tunnel.spec.peer_public_key.as_str());

            if let Ok(public_key) = key {
                if allocations.try_insert(public_key, address).is_err() {
                    troublemakers.push(tunnel);
                }
            } else {
                troublemakers.push(tunnel);
            }
        }
    }

    Some((allocations, troublemakers))
}
