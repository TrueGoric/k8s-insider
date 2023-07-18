use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ugh
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(untagged)]
pub enum IpNetFit {
    V4 { ipv4: Ipv4Net },
    V6 { ipv6: Ipv6Net },
}

impl From<IpNet> for IpNetFit {
    fn from(value: IpNet) -> Self {
        match value {
            IpNet::V4(ipv4) => Self::V4 { ipv4 },
            IpNet::V6(ipv6) => Self::V6 { ipv6 },
        }
    }
}

impl From<IpNetFit> for IpNet {
    fn from(value: IpNetFit) -> Self {
        match value {
            IpNetFit::V4 { ipv4 } => Self::V4(ipv4),
            IpNetFit::V6 { ipv6 } => Self::V6(ipv6),
        }
    }
}

impl From<&IpNetFit> for IpNet {
    fn from(value: &IpNetFit) -> Self {
        match value {
            IpNetFit::V4 { ipv4 } => Self::V4(*ipv4),
            IpNetFit::V6 { ipv6 } => Self::V6(*ipv6),
        }
    }
}
