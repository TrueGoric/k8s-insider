use std::{
    fmt::{Display, Formatter},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IpPairError {
    #[error("Couldn't parse IP address/CIDR!")]
    Invalid,
    #[error("The value contained two IPv4 addresses/CIDRs!")]
    DuplicateIpv4,
    #[error("The value contained two IPv6 addresses/CIDRs!")]
    DuplicateIpv6,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, JsonSchema)]
#[serde(untagged)]
pub enum IpAddrPair {
    Ipv4 { ipv4: Ipv4Addr },
    Ipv6 { ipv6: Ipv6Addr },
    Ipv4v6 { ipv4: Ipv4Addr, ipv6: Ipv6Addr },
}

impl Default for IpAddrPair {
    fn default() -> Self {
        Self::Ipv4 { ipv4: Ipv4Addr::from(0) } // consistent with Ipv4Net default
    }
}

impl Display for IpAddrPair {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IpAddrPair::Ipv4 { ipv4 } => f.write_fmt(format_args!("{ipv4}")),
            IpAddrPair::Ipv6 { ipv6 } => f.write_fmt(format_args!("{ipv6}")),
            IpAddrPair::Ipv4v6 { ipv4, ipv6 } => f.write_fmt(format_args!("{ipv4},{ipv6}")),
        }
    }
}

impl FromStr for IpAddrPair {
    type Err = IpPairError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.splitn(2, ',');
        let first = split
            .next()
            .ok_or(IpPairError::Invalid)?
            .parse::<IpAddr>()
            .map_err(|_| IpPairError::Invalid)?;
        let second = split.next().map(|val| val.parse::<IpAddr>());

        match first {
            IpAddr::V4(ipv4) => match second {
                Some(ipv6) => match ipv6.map_err(|_| IpPairError::Invalid)? {
                    IpAddr::V4(_) => Err(IpPairError::DuplicateIpv4),
                    IpAddr::V6(ipv6) => Ok(IpAddrPair::Ipv4v6 { ipv4, ipv6 }),
                },
                None => Ok(IpAddrPair::Ipv4 { ipv4 }),
            },
            IpAddr::V6(ipv6) => match second {
                Some(ipv4) => match ipv4.map_err(|_| IpPairError::Invalid)? {
                    IpAddr::V4(ipv4) => Ok(IpAddrPair::Ipv4v6 { ipv4, ipv6 }),
                    IpAddr::V6(_) => Err(IpPairError::DuplicateIpv6),
                },
                None => Ok(IpAddrPair::Ipv6 { ipv6 }),
            },
        }
    }
}

impl From<Ipv4Addr> for IpAddrPair {
    fn from(value: Ipv4Addr) -> Self {
        IpAddrPair::Ipv4 { ipv4: value }
    }
}

impl From<Ipv6Addr> for IpAddrPair {
    fn from(value: Ipv6Addr) -> Self {
        IpAddrPair::Ipv6 { ipv6: value }
    }
}

impl From<IpAddr> for IpAddrPair {
    fn from(value: IpAddr) -> Self {
        match value {
            IpAddr::V4(ipv4) => IpAddrPair::Ipv4 { ipv4 },
            IpAddr::V6(ipv6) => IpAddrPair::Ipv6 { ipv6 },
        }
    }
}

impl From<IpAddrPair> for Vec<String> {
    fn from(value: IpAddrPair) -> Self {
        match value {
            IpAddrPair::Ipv4 { ipv4 } => vec![ipv4.to_string()],
            IpAddrPair::Ipv6 { ipv6 } => vec![ipv6.to_string()],
            IpAddrPair::Ipv4v6 { ipv4, ipv6 } => vec![ipv4.to_string(), ipv6.to_string()],
        }
    }
}

impl From<IpAddrPair> for Vec<IpAddr> {
    fn from(value: IpAddrPair) -> Self {
        match value {
            IpAddrPair::Ipv4 { ipv4 } => vec![IpAddr::V4(ipv4)],
            IpAddrPair::Ipv6 { ipv6 } => vec![IpAddr::V6(ipv6)],
            IpAddrPair::Ipv4v6 { ipv4, ipv6 } => vec![IpAddr::V4(ipv4), IpAddr::V6(ipv6)],
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, JsonSchema)]
#[serde(untagged)]
pub enum IpNetPair {
    Ipv4 { netv4: Ipv4Net },
    Ipv6 { netv6: Ipv6Net },
    Ipv4v6 { netv4: Ipv4Net, netv6: Ipv6Net },
}

impl IpNetPair {
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

pub trait Contains<T> {
    fn contains(&self, other: &T) -> bool;
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
