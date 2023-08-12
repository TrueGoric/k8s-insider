use k8s_insider_core::wireguard::keys::{InvalidWgKey, WgKey};
use k8s_openapi::serde::{Deserialize, Serialize};

use super::network::NetworkIdentifier;

#[derive(Serialize, Deserialize, Clone, Default, Debug, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TunnelIdentifier {
    pub network: NetworkIdentifier,
    pub name: String,
}

impl TunnelIdentifier {
    pub fn from_network_identifier(network: NetworkIdentifier, name: String) -> Self {
        Self { network, name }
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TunnelConfig {
    pub name: String,
    pub private_key: String,
}

impl TunnelConfig {
    pub fn new(name: String, private_key: WgKey) -> Self {
        Self {
            name,
            private_key: private_key.to_base64(),
        }
    }

    pub fn try_get_wgkey(&self) -> Result<WgKey, InvalidWgKey> {
        WgKey::from_base64(&self.private_key)
    }
}
