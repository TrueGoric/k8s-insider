use std::{net::{Ipv4Addr, Ipv6Addr, IpAddr}, fmt::{Display, Formatter}, str::FromStr};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::IpPairError;

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
