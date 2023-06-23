use std::{env::var, net::IpAddr};

use derive_builder::Builder;
use ipnet::IpNet;
use kube::core::ObjectMeta;

use crate::FromEnvError;

use super::labels::get_controller_labels;

pub mod configmap;
pub mod deployment;
pub mod rbac;

#[derive(Debug, Builder)]
pub struct ControllerRelease {
    pub namespace: String,
    pub kube_dns: Option<IpAddr>,
    pub service_domain: Option<String>,
    pub service_cidr: IpNet,
    pub pod_cidr: IpNet,
    pub controller_image_name: String,
    pub tunnel_image_name: String,
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
            controller_image_name: var("KUBE_INSIDER_AGENT_IMAGE_NAME")
                .map_err(FromEnvError::Var)?,
            tunnel_image_name: var("KUBE_INSIDER_TUNNEL_IMAGE_NAME").map_err(FromEnvError::Var)?,
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
