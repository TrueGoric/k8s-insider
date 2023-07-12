use std::{
    collections::{BTreeSet, HashMap},
    fmt::Display,
    net::{Ipv4Addr, Ipv6Addr},
    ops::{Deref, DerefMut},
    process::exit,
};

use ipnet::{IpAdd, Ipv4Net, Ipv6Net};
use k8s_insider_core::{
    ip::{addrpair::DualStackTryGet, range::UniqueRandomWrappingHostsIpIterator, Contains},
    kubernetes::operations::{list_resources, try_remove_resource},
    resources::{crd::v1alpha1::tunnel::Tunnel, router::RouterRelease},
    wireguard::keys::WgKey,
    AsPrimitive, FromPrimitive, Unsigned,
};
use kube::{
    api::{DeleteParams, ListParams},
    Client,
};
use log::{error, info, warn};
use thiserror::Error;
use tokio::sync::RwLock;

pub type Ipv4Allocations = Allocations<Ipv4Addr, Ipv4Net, u32>;

#[derive(Debug)]
pub struct Allocations<IP, IPNet, IPSize> {
    range: IPNet,
    allocations: HashMap<WgKey, IP>,
    reserved: BTreeSet<IP>,
    subnet_iterator: UniqueRandomWrappingHostsIpIterator<IP, IPSize>,
}

impl<IP, IPNet, IPSize> Allocations<IP, IPNet, IPSize>
where
    IP: Copy + Clone + Ord + Display + IpAdd<IPSize, Output = IP> + PartialEq<IP>,
    IPNet: Contains<IP>,
    IPSize: Unsigned + FromPrimitive + AsPrimitive<u128>,
{
    pub fn new(
        range: IPNet,
        subnet_iterator: UniqueRandomWrappingHostsIpIterator<IP, IPSize>,
    ) -> Self {
        Self {
            range,
            allocations: HashMap::new(),
            reserved: BTreeSet::new(),
            subnet_iterator,
        }
    }

    pub fn try_insert(&mut self, key: WgKey, ip: IP) -> Result<IP, AllocationsError<IP>> {
        if self.allocations.contains_key(&key) {
            return Err(AllocationsError::WgKeyConflict(key));
        }

        if self.reserved.contains(&ip) {
            return Err(AllocationsError::IpConflict(ip));
        }

        if !self.is_in_range(&ip) {
            return Err(AllocationsError::IpOutOfRange(ip));
        }

        if self.allocations.insert(key, ip).is_some() {
            panic!("Allocation insert returned a value that wasn't supposed to be there!");
        }

        if !self.reserved.insert(ip) {
            panic!("Reserved insertion reports an IP that shouldn't be there!");
        }

        info!("Allocated {ip} address!");

        Ok(ip)
    }

    pub fn try_allocate(&mut self, key: WgKey) -> Result<IP, AllocationsError<IP>> {
        if self.allocations.contains_key(&key) {
            return Err(AllocationsError::WgKeyConflict(key));
        }

        let mut allocated = false;
        let mut ip = self.subnet_iterator.get();

        for _ in 1..self.subnet_iterator.address_count() {
            if self.reserved.insert(ip) {
                allocated = true;
                break;
            }

            ip = self.subnet_iterator.get();
        }

        if !allocated {
            return Err(AllocationsError::RangeExhausted);
        }

        if self.allocations.insert(key, ip).is_some() {
            panic!("Allocation insert returned a value that wasn't supposed to be there!");
        }

        info!("Allocated {ip} address!");

        Ok(ip)
    }

    pub fn try_remove(&mut self, key: &WgKey) -> Option<IP> {
        if let Some(ip) = self.allocations.remove(key) {
            self.reserved.remove(&ip);

            info!("Deallocated {ip} address!");

            return Some(ip);
        }

        None
    }

    pub fn is_in_range(&self, ip: &IP) -> bool {
        self.range.contains(ip)
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

pub type Ipv4AllocationsSync = AllocationsSync<Ipv4Addr, Ipv4Net, u32>;
pub type Ipv6AllocationsSync = AllocationsSync<Ipv6Addr, Ipv6Net, u128>;

pub struct AllocationsSync<IP, IPNet, IPSize>(RwLock<Allocations<IP, IPNet, IPSize>>);

impl<IP, IPNet, IPSize> AllocationsSync<IP, IPNet, IPSize>
where
    IP: Copy + Clone + Ord + Display + IpAdd<IPSize, Output = IP> + PartialEq<IP>,
    IPNet: Contains<IP>,
    IPSize: Unsigned + FromPrimitive + AsPrimitive<u128>,
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

    pub async fn get_or_insert<F: FnOnce() -> IP>(
        &self,
        key: &WgKey,
        ip_getter: F,
    ) -> Result<IP, AllocationsError<IP>> {
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

    pub async fn try_remove(&self, key: &WgKey) -> Option<IP> {
        self.write().await.try_remove(key)
    }
}

impl<IP, IPNet, IPSize> Deref for AllocationsSync<IP, IPNet, IPSize> {
    type Target = RwLock<Allocations<IP, IPNet, IPSize>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<IP, IPNet, IPSize> DerefMut for AllocationsSync<IP, IPNet, IPSize> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<IP, IPNet, IPSize> From<Allocations<IP, IPNet, IPSize>>
    for AllocationsSync<IP, IPNet, IPSize>
{
    fn from(value: Allocations<IP, IPNet, IPSize>) -> Self {
        Self(RwLock::new(value))
    }
}

pub async fn sync_allocations(
    client: &Client,
    router_release: &RouterRelease,
) -> Result<(Option<Ipv4AllocationsSync>, Option<Ipv6AllocationsSync>), kube::Error> {
    info!("Synchronizing address allocations...");

    let tunnels = list_resources(client, &router_release.namespace, &ListParams::default()).await?;

    let (allocations_ipv4, conflicting_tunnels) =
        match init_ipv4_allocations(router_release, tunnels.iter()) {
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

        warn!(
            "Removing '{tunnel_name}' due to conflicting IP address ({:?})! Someone was naughty!",
            tunnel.status.as_ref().and_then(|s| s.address)
        );

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
    let peer_cidr = release.peer_cidr.try_get_ipv4()?;
    let iterator = UniqueRandomWrappingHostsIpIterator::new(peer_cidr);
    let mut allocations = Ipv4Allocations::new(peer_cidr, iterator);
    let mut troublemakers = Vec::new();

    allocations
        .try_insert(
            release.server_keys.get_public_key().to_owned(),
            release.router_ip.try_get_ipv4()?,
        )
        .unwrap();

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
