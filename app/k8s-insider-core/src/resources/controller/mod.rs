use k8s_openapi::api::core::v1::ConfigMap;
use kube::core::ObjectMeta;
use std::{borrow::Cow, env::var};
use thiserror::Error;

use crate::ip::{addrpair::IpAddrPair, netpair::IpNetPair, IpPairError};

use super::labels::get_controller_labels;

pub mod configmap;
pub mod deployment;
pub mod rbac;

pub const CONTROLLER_RELEASE_NAME: &str = "k8s-insider-controller";

#[derive(Debug, Clone)]
pub struct ControllerRelease {
    pub namespace: String,
    pub kube_dns: Option<IpAddrPair>,
    pub service_domain: Option<String>,
    pub service_cidr: IpNetPair,
    pub pod_cidr: IpNetPair,
    pub controller_image_name: String,
    pub controller_image_tag: String,
    pub network_manager_image_name: String,
    pub network_manager_image_tag: String,
    pub router_image_name: String,
    pub router_image_tag: String,
}

#[derive(Debug, Error)]
pub enum FromError {
    #[error("Env var unavailable: {}", .0)]
    VarUnset(std::env::VarError),
    #[error("ConfigMap data is unset!")]
    MissingData,
    #[error("ConfigMap key unavailable: {}", .0)]
    MissingKey(Cow<'static, str>),
    #[error("IP address couldn't be parsed: {}", .0)]
    IpAddrParse(IpPairError),
    #[error("IP CIDR couldn't be parsed: {}", .0)]
    IpNetParse(IpPairError),
}

impl ControllerRelease {
    pub fn from_env() -> Result<Self, FromError> {
        Ok(Self {
            namespace: var("KUBE_INSIDER_NAMESPACE").map_err(FromError::VarUnset)?,
            kube_dns: match var("KUBE_INSIDER_DNS") {
                Ok(ip) => Some(ip.parse().map_err(FromError::IpAddrParse)?),
                Err(_) => None,
            },
            service_domain: var("KUBE_INSIDER_SERVICE_DOMAIN").ok(),
            service_cidr: var("KUBE_INSIDER_SERVICE_CIDR")
                .map_err(FromError::VarUnset)?
                .parse()
                .map_err(FromError::IpNetParse)?,
            pod_cidr: var("KUBE_INSIDER_POD_CIDR")
                .map_err(FromError::VarUnset)?
                .parse()
                .map_err(FromError::IpNetParse)?,
            controller_image_name: var("KUBE_INSIDER_CONTROLLER_IMAGE_NAME")
                .map_err(FromError::VarUnset)?,
            controller_image_tag: var("KUBE_INSIDER_CONTROLLER_IMAGE_TAG")
                .map_err(FromError::VarUnset)?,
            network_manager_image_name: var("KUBE_INSIDER_NETWORK_MANAGER_IMAGE_NAME")
                .map_err(FromError::VarUnset)?,
            network_manager_image_tag: var("KUBE_INSIDER_NETWORK_MANAGER_IMAGE_TAG")
                .map_err(FromError::VarUnset)?,
            router_image_name: var("KUBE_INSIDER_ROUTER_IMAGE_NAME")
                .map_err(FromError::VarUnset)?,
            router_image_tag: var("KUBE_INSIDER_ROUTER_IMAGE_TAG")
                .map_err(FromError::VarUnset)?,
        })
    }

    pub fn from_configmap(configmap: &ConfigMap) -> Result<Self, FromError> {
        let data = configmap.data.as_ref().ok_or(FromError::MissingData)?;

        Ok(Self {
            namespace: data
                .get("KUBE_INSIDER_NAMESPACE")
                .ok_or(FromError::MissingKey("KUBE_INSIDER_NAMESPACE".into()))?
                .to_owned(),
            kube_dns: match data.get("KUBE_INSIDER_DNS") {
                Some(ip) => Some(ip.parse().map_err(FromError::IpAddrParse)?),
                None => None,
            },
            service_domain: data
                .get("KUBE_INSIDER_SERVICE_DOMAIN")
                .map(|v| v.to_owned()),
            service_cidr: data
                .get("KUBE_INSIDER_SERVICE_CIDR")
                .ok_or(FromError::MissingKey("KUBE_INSIDER_SERVICE_CIDR".into()))?
                .parse()
                .map_err(FromError::IpNetParse)?,
            pod_cidr: data
                .get("KUBE_INSIDER_POD_CIDR")
                .ok_or(FromError::MissingKey("KUBE_INSIDER_POD_CIDR".into()))?
                .parse()
                .map_err(FromError::IpNetParse)?,
            controller_image_name: data
                .get("KUBE_INSIDER_CONTROLLER_IMAGE_NAME")
                .ok_or(FromError::MissingKey(
                    "KUBE_INSIDER_CONTROLLER_IMAGE_NAME".into(),
                ))?
                .to_owned(),
            controller_image_tag: data
                .get("KUBE_INSIDER_CONTROLLER_IMAGE_TAG")
                .ok_or(FromError::MissingKey(
                    "KUBE_INSIDER_CONTROLLER_IMAGE_TAG".into(),
                ))?
                .to_owned(),
            network_manager_image_name: data
                .get("KUBE_INSIDER_NETWORK_MANAGER_IMAGE_NAME")
                .ok_or(FromError::MissingKey(
                    "KUBE_INSIDER_NETWORK_MANAGER_IMAGE_NAME".into(),
                ))?
                .to_owned(),
            network_manager_image_tag: data
                .get("KUBE_INSIDER_NETWORK_MANAGER_IMAGE_TAG")
                .ok_or(FromError::MissingKey(
                    "KUBE_INSIDER_NETWORK_MANAGER_IMAGE_TAG".into(),
                ))?
                .to_owned(),
            router_image_name: data
                .get("KUBE_INSIDER_ROUTER_IMAGE_NAME")
                .ok_or(FromError::MissingKey(
                    "KUBE_INSIDER_ROUTER_IMAGE_NAME".into(),
                ))?
                .to_owned(),
            router_image_tag: data
                .get("KUBE_INSIDER_ROUTER_IMAGE_TAG")
                .ok_or(FromError::MissingKey(
                    "KUBE_INSIDER_ROUTER_IMAGE_TAG".into(),
                ))?
                .to_owned(),
        })
    }

    pub fn generate_default_metadata(&self) -> ObjectMeta {
        self.generate_metadata(CONTROLLER_RELEASE_NAME)
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
        self.generate_clusterwide_metadata(CONTROLLER_RELEASE_NAME)
    }

    pub fn generate_clusterwide_metadata(&self, name: &str) -> ObjectMeta {
        ObjectMeta {
            labels: Some(get_controller_labels()),
            name: Some(name.to_owned()),
            ..Default::default()
        }
    }

    pub fn get_controller_image(&self) -> String {
        format!("{}:{}", self.controller_image_name, self.controller_image_tag)
    }

    pub fn get_network_manager_image(&self) -> String {
        format!("{}:{}", self.network_manager_image_name, self.network_manager_image_tag)
    }

    pub fn get_router_image(&self) -> String {
        format!("{}:{}", self.router_image_name, self.router_image_tag)
    }
}
