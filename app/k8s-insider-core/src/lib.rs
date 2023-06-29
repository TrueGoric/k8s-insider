pub mod detectors;
pub mod generators;
pub mod helpers;
pub mod ippair;
pub mod kubernetes;
pub mod resources;
pub mod tunnel_info;
pub mod wireguard;

pub const RESOURCE_GROUP: &str = "k8s-insider.dev";

pub const CONTROLLER_CLUSTERROLE_NAME: &str = "k8s-insider-controller";
pub const ROUTER_CLUSTERROLE_NAME: &str = "k8s-insider-router";
