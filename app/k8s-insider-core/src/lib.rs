pub mod detectors;
pub mod generators;
pub mod helpers;
pub mod ip;
pub mod kubernetes;
pub mod resources;
pub mod tunnel_info;
pub mod wireguard;

pub use num_traits::{AsPrimitive, FromPrimitive, Unsigned};

pub const RESOURCE_GROUP: &str = "k8s-insider.dev";

pub const CONTROLLER_CLUSTERROLE_NAME: &str = "k8s-insider-controller";
pub const NETWORK_MANAGER_CLUSTERROLE_NAME: &str = "k8s-insider-network-manager";
pub const ROUTER_CLUSTERROLE_NAME: &str = "k8s-insider-router";
