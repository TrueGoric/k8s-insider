use std::str::FromStr;

use chrono::DateTime;
use k8s_insider_core::tunnel_info::handshakes::HandshakeInfo;

use warp::hyper::{Response, StatusCode};
use wireguard_control::InterfaceName;

pub(crate) trait HandshakeResponse {
    fn response(self) -> Response<String>;
}

impl HandshakeResponse for Result<Vec<HandshakeInfo>, std::io::Error> {
    fn response(self) -> Response<String> {
        Response::builder()
            .status(match &self {
                Ok(_) => StatusCode::OK,
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            })
            .body(match self {
                Ok(val) => serde_json::to_string(&val).unwrap(),
                Err(err) => format!("\'{err}\'"),
            })
            .unwrap()
    }
}

pub fn get_handshakes(interface: &str) -> Result<Vec<HandshakeInfo>, std::io::Error> {
    let interface = InterfaceName::from_str(interface).unwrap();
    let handshakes =
        wireguard_control::Device::get(&interface, wireguard_control::Backend::Kernel)?
            .peers
            .iter()
            .map(|peer| HandshakeInfo {
                public_key: peer.config.public_key.to_base64(),
                last_handshake: peer.stats.last_handshake_time.map(|h| DateTime::from(h)),
            })
            .collect();

    Ok(handshakes)
}
