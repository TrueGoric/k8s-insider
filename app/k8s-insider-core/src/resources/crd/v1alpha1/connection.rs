use chrono::{DateTime, Utc};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    group = "k8s-insider.dev",
    version = "v1alpha1",
    kind = "Connection",
    namespaced,
    status = "ConnectionStatus"
)]
pub struct ConnectionSpec {
    /// peer public key
    pub peer_public_key: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct ConnectionStatus {
    /// last handshake
    pub last_handshake: Option<DateTime<Utc>>,
}