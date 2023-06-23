use thiserror::Error;

pub mod detectors;
pub mod generators;
pub mod helpers;
pub mod kubernetes;
pub mod tunnel_info;
pub mod resources;

pub const RESOURCE_GROUP: &str = "k8s-insider.dev";
pub const FIELD_MANAGER: &str = "k8s-insider";

pub const CONTROLLER_CLUSTERROLE_NAME: &str = "k8s-insider-controller";
pub const ROUTER_CLUSTERROLE_NAME: &str = "k8s-insider-router";

#[derive(Debug, Error)]
pub enum FromEnvError {
    #[error("Env var unavailable: {}", .0)]
    Var(std::env::VarError),
    #[error("IP address couldn't be parsed: {}", .0)]
    IpAddrParse(std::net::AddrParseError),
    #[error("IP CIDR couldn't be parsed: {}", .0)]
    IpNetParse(ipnet::AddrParseError),
}
