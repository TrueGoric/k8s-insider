use k8s_insider_core::wireguard::keys::{WgKey, InvalidWgKey};
use k8s_openapi::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelConfig {
    pub context: String,
    pub name: String,
    pub namespace: String,
    pub private_key: String,
}

impl TunnelConfig {
    pub fn new(context: String, name: String, namespace: String, private_key: WgKey) -> Self {
        Self {
            context,
            name,
            namespace,
            private_key: private_key.to_base64(),
        }
    }

    pub fn try_get_wgkey(&self) -> Result<WgKey, InvalidWgKey> {
        WgKey::from_base64(&self.private_key)
    }
}
