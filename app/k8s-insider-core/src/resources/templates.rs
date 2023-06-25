use crate::ippair::IpAddrPair;

pub fn server_interface_template(ip: &IpAddrPair, port: u32, private_key: &str) -> String {
    format!(
        "[Interface]
    Address = {ip}
    ListenPort = {port}
    PrivateKey = {private_key}"
    )
}

pub fn peer_interface_template(ip: &IpAddrPair, port: u32, private_key: &str, dns: &str) -> String {
    format!(
        "[Interface]
    Address = {ip}
    ListenPort = {port}
    PrivateKey = {private_key}
    DNS = {dns}"
    )
}
