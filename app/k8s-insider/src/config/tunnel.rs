use k8s_insider_core::wireguard::keys::WgKey;
use k8s_openapi::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelConfig {
    pub name: String,
    pub namespace: String,
    pub private_key: String,
}

impl TunnelConfig {
    pub fn new(name: String, namespace: String, private_key: WgKey) -> Self {
        Self {
            name,
            namespace,
            private_key: private_key.to_base64(),
        }
    }
}
