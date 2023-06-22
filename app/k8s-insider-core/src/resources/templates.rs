use std::net::IpAddr;

pub fn server_interface_template(ip: &IpAddr, port: u32, private_key: &str) -> String {
    format!(
"[Interface]
    Address = {ip}
    ListenPort = {port}
    PrivateKey = {private_key}")
}

pub fn peer_interface_template(ip: &IpAddr, port: u32, private_key: &str, dns: &str) -> String {
    format!(
"[Interface]
    Address = {ip}
    ListenPort = {port}
    PrivateKey = {private_key}
    DNS = {dns}")
}