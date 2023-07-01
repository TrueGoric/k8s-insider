use kube::core::ObjectMeta;
use std::env::var;
use thiserror::Error;

use crate::ip::{addrpair::IpAddrPair, netpair::IpNetPair, IpPairError};

use super::labels::get_controller_labels;

pub mod configmap;
pub mod deployment;
pub mod rbac;

#[derive(Debug, Clone)]
pub struct ControllerRelease {
    pub namespace: String,
    pub kube_dns: Option<IpAddrPair>,
    pub service_domain: Option<String>,
    pub service_cidr: IpNetPair,
    pub pod_cidr: IpNetPair,
    pub controller_image_name: String,
    pub router_image_name: String,
}

#[derive(Debug, Error)]
pub enum FromEnvError {
    #[error("Env var unavailable: {}", .0)]
    Var(std::env::VarError),
    #[error("IP address couldn't be parsed: {}", .0)]
    IpAddrParse(IpPairError),
    #[error("IP CIDR couldn't be parsed: {}", .0)]
    IpNetParse(IpPairError),
}

impl ControllerRelease {
    pub fn from_env() -> Result<Self, FromEnvError> {
        Ok(Self {
            namespace: var("KUBE_INSIDER_NAMESPACE").map_err(FromEnvError::Var)?,
            kube_dns: match var("KUBE_INSIDER_DNS") {
                Ok(ip) => Some(ip.parse().map_err(FromEnvError::IpAddrParse)?),
                Err(_) => None,
            },
            service_domain: var("KUBE_INSIDER_SERVICE_DOMAIN").ok(),
            service_cidr: var("KUBE_INSIDER_SERVICE_CIDR")
                .map_err(FromEnvError::Var)?
                .parse()
                .map_err(FromEnvError::IpNetParse)?,
            pod_cidr: var("KUBE_INSIDER_POD_CIDR")
                .map_err(FromEnvError::Var)?
                .parse()
                .map_err(FromEnvError::IpNetParse)?,
            controller_image_name: var("KUBE_INSIDER_CONTROLLER_IMAGE_NAME")
                .map_err(FromEnvError::Var)?,
            router_image_name: var("KUBE_INSIDER_ROUTER_IMAGE_NAME").map_err(FromEnvError::Var)?,
        })
    }

    pub fn generate_default_metadata(&self) -> ObjectMeta {
        self.generate_metadata("k8s-insider-controller")
    }

    pub fn generate_metadata(&self, name: &str) -> ObjectMeta {
        ObjectMeta {
            labels: Some(get_controller_labels()),
            namespace: Some(self.namespace.to_owned()),
            name: Some(name.to_owned()),
            ..Default::default()
        }
    }

    pub fn generate_clusterwide_default_metadata(&self) -> ObjectMeta {
        self.generate_clusterwide_metadata("k8s-insider-controller")
    }

    pub fn generate_clusterwide_metadata(&self, name: &str) -> ObjectMeta {
        ObjectMeta {
            labels: Some(get_controller_labels()),
            name: Some(name.to_owned()),
            ..Default::default()
        }
    }
}
