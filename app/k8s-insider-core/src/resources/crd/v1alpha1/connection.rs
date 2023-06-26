use chrono::{DateTime, Utc};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use serde_with::skip_serializing_none;

#[derive(CustomResource, Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[kube(
    group = "k8s-insider.dev",
    version = "v1alpha1",
    kind = "Connection",
    namespaced,
    status = "ConnectionStatus",
    derive = "Default"
)]
pub struct ConnectionSpec {
    /// peer public key
    pub peer_public_key: String,
}

#[skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatus {
    /// last handshake
    pub last_handshake: Option<DateTime<Utc>>,
}