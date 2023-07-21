use k8s_insider_core::wireguard::keys::{InvalidWgKey, WgKey};
use k8s_openapi::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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
