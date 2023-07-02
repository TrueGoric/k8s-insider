use std::{fmt::{Display, Formatter}, str::FromStr};

use ipnet::{Ipv4Net, Ipv6Net, IpNet};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{addrpair::IpAddrPair, Contains, IpPairError};

#[derive(Deserialize, Serialize, Clone, Copy, Debug, JsonSchema)]
#[serde(untagged)]
pub enum IpNetPair {
    Ipv4 { netv4: Ipv4Net },
    Ipv6 { netv6: Ipv6Net },
    Ipv4v6 { netv4: Ipv4Net, netv6: Ipv6Net },
}

impl IpNetPair {
    pub fn iter<'a>(&'a self) -> IpNetPairIter<'_> {
        IpNetPairIter::<'a>::new(self)
    }

    pub fn try_get_ipv4(&self) -> Option<Ipv4Net> {
        match self {
            IpNetPair::Ipv4 { netv4 } => Some(*netv4),
            IpNetPair::Ipv4v6 { netv4, .. } => Some(*netv4),
            _ => None
        }
    }

    pub fn try_get_ipv6(&self) -> Option<Ipv6Net> {
        match self {
            IpNetPair::Ipv6 { netv6 } => Some(*netv6),
            IpNetPair::Ipv4v6 { netv6, .. } => Some(*netv6),
            _ => None
        }
    }

    pub fn trunc(&self) -> Self {
        match self {
            Self::Ipv4 { netv4 } => Self::Ipv4 { netv4: netv4.trunc() },
            Self::Ipv6 { netv6 } => Self::Ipv6 { netv6: netv6.trunc() },
            Self::Ipv4v6 { netv4, netv6 } => Self::Ipv4v6 {
                netv4: netv4.trunc(),
                netv6: netv6.trunc(),
            },
        }
    }

    pub fn first_addresses(&self) -> IpAddrPair {
        match self {
            IpNetPair::Ipv4 { netv4 } => netv4.hosts().next().unwrap().into(),
            IpNetPair::Ipv6 { netv6 } => netv6.hosts().next().unwrap().into(),
            IpNetPair::Ipv4v6 { netv4, netv6 } => IpAddrPair::Ipv4v6 {
                ipv4: netv4.hosts().next().unwrap(),
                ipv6: netv6.hosts().next().unwrap(),
            },
        }
    }
}


impl Contains<IpAddrPair> for IpNetPair {
    fn contains(&self, other: &IpAddrPair) -> bool {
        match self {
            IpNetPair::Ipv4 { netv4 } => match other {
                IpAddrPair::Ipv4 { ipv4 } => netv4.contains(ipv4),
                _ => false,
            },
            IpNetPair::Ipv6 { netv6 } => match other {
                IpAddrPair::Ipv6 { ipv6 } => netv6.contains(ipv6),
                _ => false,
            },
            IpNetPair::Ipv4v6 { netv4, netv6 } => match other {
                IpAddrPair::Ipv4 { ipv4 } => netv4.contains(ipv4),
                IpAddrPair::Ipv6 { ipv6 } => netv6.contains(ipv6),
                IpAddrPair::Ipv4v6 { ipv4, ipv6 } => netv4.contains(ipv4) && netv6.contains(ipv6),
            },
        }
    }
}

impl Default for IpNetPair {
    fn default() -> Self {
        Self::Ipv4 { netv4: Ipv4Net::default() }
    }
}

impl Display for IpNetPair {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IpNetPair::Ipv4 { netv4 } => f.write_fmt(format_args!("{netv4}")),
            IpNetPair::Ipv6 { netv6 } => f.write_fmt(format_args!("{netv6}")),
            IpNetPair::Ipv4v6 { netv4, netv6 } => f.write_fmt(format_args!("{netv4},{netv6}")),
        }
    }
}

impl FromStr for IpNetPair {
    type Err = IpPairError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.splitn(2, ',');
        let first = split
            .next()
            .ok_or(IpPairError::Invalid)?
            .parse::<IpNet>()
            .map_err(|_| IpPairError::Invalid)?;
        let second = split.next().map(|val| val.parse::<IpNet>());

        match first {
            IpNet::V4(netv4) => match second {
                Some(netv6) => match netv6.map_err(|_| IpPairError::Invalid)? {
                    IpNet::V4(_) => Err(IpPairError::DuplicateIpv4),
                    IpNet::V6(netv6) => Ok(IpNetPair::Ipv4v6 { netv4, netv6 }),
                },
                None => Ok(IpNetPair::Ipv4 { netv4 }),
            },
            IpNet::V6(netv6) => match second {
                Some(netv4) => match netv4.map_err(|_| IpPairError::Invalid)? {
                    IpNet::V4(netv4) => Ok(IpNetPair::Ipv4v6 { netv4, netv6 }),
                    IpNet::V6(_) => Err(IpPairError::DuplicateIpv6),
                },
                None => Ok(IpNetPair::Ipv6 { netv6 }),
            },
        }
    }
}

impl From<Ipv4Net> for IpNetPair {
    fn from(value: Ipv4Net) -> Self {
        IpNetPair::Ipv4 { netv4: value }
    }
}

impl From<Ipv6Net> for IpNetPair {
    fn from(value: Ipv6Net) -> Self {
        IpNetPair::Ipv6 { netv6: value }
    }
}

impl From<IpNet> for IpNetPair {
    fn from(value: IpNet) -> Self {
        match value {
            IpNet::V4(netv4) => IpNetPair::Ipv4 { netv4 },
            IpNet::V6(netv6) => IpNetPair::Ipv6 { netv6 },
        }
    }
}

impl From<IpNetPair> for Vec<IpNet> {
    fn from(value: IpNetPair) -> Self {
        match value {
            IpNetPair::Ipv4 { netv4 } => vec![IpNet::V4(netv4)],
            IpNetPair::Ipv6 { netv6 } => vec![IpNet::V6(netv6)],
            IpNetPair::Ipv4v6 { netv4, netv6 } => vec![IpNet::V4(netv4), IpNet::V6(netv6)],
        }
    }
}

impl From<IpNetPair> for Vec<String> {
    fn from(value: IpNetPair) -> Self {
        match value {
            IpNetPair::Ipv4 { netv4 } => vec![netv4.to_string()],
            IpNetPair::Ipv6 { netv6 } => vec![netv6.to_string()],
            IpNetPair::Ipv4v6 { netv4, netv6 } => vec![netv4.to_string(), netv6.to_string()],
        }
    }
}

pub struct IpNetPairIter<'a> {
    pub pair: &'a IpNetPair,
    first_served: bool,
}

impl<'a> IpNetPairIter<'a> {
    pub fn new(pair: &'a IpNetPair) -> Self {
        Self {
            pair, first_served: false
        }
    }
}

impl<'a> Iterator for IpNetPairIter<'a> {
    type Item = IpNet;

    fn next(&mut self) -> Option<Self::Item> {
        match self.first_served {
            true => match self.pair {
                IpNetPair::Ipv4v6 { netv6, ..} => Some(IpNet::V6(*netv6)),
                _ => None
            },
            false => {
                self.first_served = true;

                match self.pair {
                    IpNetPair::Ipv4 { netv4 } => Some(IpNet::V4(*netv4)),
                    IpNetPair::Ipv6 { netv6 } => Some(IpNet::V6(*netv6)),
                    IpNetPair::Ipv4v6 { netv4, ..} => Some(IpNet::V4(*netv4)),
                }
            },
        }
    }
}