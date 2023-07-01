use std::net::{Ipv4Addr, Ipv6Addr};

use ipnet::{Ipv4Net, Ipv6Net};
use thiserror::Error;

pub mod addrpair;
pub mod netpair;

#[derive(Debug, Error)]
pub enum IpPairError {
    #[error("Couldn't parse IP address/CIDR!")]
    Invalid,
    #[error("The value contained two IPv4 addresses/CIDRs!")]
    DuplicateIpv4,
    #[error("The value contained two IPv6 addresses/CIDRs!")]
    DuplicateIpv6,
}

pub trait Contains<T> {
    fn contains(&self, other: &T) -> bool;
}

impl Contains<Ipv4Addr> for Ipv4Net {
    fn contains(&self, other: &Ipv4Addr) -> bool {
        self.contains(other)
    }
}

impl Contains<Ipv6Addr> for Ipv6Net {
    fn contains(&self, other: &Ipv6Addr) -> bool {
        self.contains(other)
    }
}
