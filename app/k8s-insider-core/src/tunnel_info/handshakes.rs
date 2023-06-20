use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HandshakeInfo {
    pub public_key: String,
    pub last_handshake: Option<DateTime<Utc>>
}