use std::net::IpAddr;

use derive_builder::Builder;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::core::ObjectMeta;
use thiserror::Error;
use wireguard_control::{InvalidKey, Key};

use crate::{
    helpers::AndIfSome,
    ippair::{Contains, IpAddrPair, IpNetPair},
    wireguard::keys::{Keys, WgKey},
};

use super::{
    controller::ControllerRelease,
    crd::v1alpha1::network::{Network, NetworkService},
    labels::get_router_labels,
    ResourceGenerationError,
};

pub mod deployment;
pub mod rbac;
pub mod secret;
pub mod service;

#[derive(Debug, Builder)]
pub struct RouterRelease {
    pub name: String,
    pub namespace: String,
    pub kube_dns: Option<IpAddrPair>,
    pub service_domain: Option<String>,
    pub service_cidr: IpNetPair,
    pub pod_cidr: IpNetPair,
    pub agent_image_name: String,
    pub tunnel_image_name: String,

    pub server_keys: Option<Keys>,
    pub peer_cidr: IpNetPair,
    pub router_ip: IpAddrPair,
    pub service: Option<RouterReleaseService>,

    pub owner: OwnerReference,
}

#[derive(Debug, Clone)]
pub enum RouterReleaseService {
    ClusterIp {
        ip: Option<IpAddrPair>,
    },
    NodePort {
        cluster_ip: Option<IpAddrPair>,
        predefined_ips: Option<Vec<IpAddr>>,
    },
    LoadBalancer {
        cluster_ip: Option<IpAddrPair>,
    },
    ExternalIp {
        cluster_ip: Option<IpAddrPair>,
        ips: Vec<IpAddr>,
    },
}

#[derive(Debug, Error)]
pub enum RouterReleaseValidationError {
    #[error("Router IP is not part of the peer network CIDR!")]
    RouterIpOutOfBounds,
    #[error("Invalid server private key!")]
    MissingKeys,
}

impl RouterReleaseBuilder {
    pub fn with_controller(&mut self, controller_release: &ControllerRelease) -> &mut Self {
        self.kube_dns(controller_release.kube_dns)
            .service_domain(controller_release.service_domain.to_owned())
            .service_cidr(controller_release.service_cidr)
            .pod_cidr(controller_release.pod_cidr)
            .agent_image_name(controller_release.controller_image_name.to_owned())
            .tunnel_image_name(controller_release.router_image_name.to_owned())
    }

    pub fn with_network_crd(
        &mut self,
        crd: &Network,
    ) -> Result<&mut Self, ResourceGenerationError> {
        let server_public_key = crd
            .status
            .as_ref()
            .and_then(|status| status.server_public_key.as_ref())
            .map(|key| Ok(WgKey::from(Key::from_base64(key)?)))
            .transpose() // I love that I can do this
            .map_err(|_: InvalidKey| {
                ResourceGenerationError::DependentInvalidData("server_public_key".into())
            })?;

        Ok(self
            .name(
                crd.metadata
                    .name
                    .as_ref()
                    .ok_or(ResourceGenerationError::DependentMissingMetadataName)?
                    .to_owned(),
            )
            .namespace(
                crd.metadata
                    .namespace
                    .as_ref()
                    .ok_or(ResourceGenerationError::DependentMissingMetadataNamespace)?
                    .to_owned(),
            )
            .peer_cidr(crd.spec.peer_cidr.to_owned())
            .router_ip(crd.spec.peer_cidr.first_addresses())
            .service(
                crd.spec
                    .network_service
                    .as_ref()
                    .map(|service| service.clone().into()),
            )
            .and_if_some(
                || server_public_key,
                |builder, server_public_key| {
                    builder.server_keys(Some(Keys::Public(server_public_key)))
                },
            ))
    }

    pub fn without_server_keys(&mut self) -> &mut Self {
        self.server_keys(None)
    }
}

impl RouterRelease {
    pub fn validated(self) -> Result<Self, RouterReleaseValidationError> {
        if !self.peer_cidr.contains(&self.router_ip) {
            return Err(RouterReleaseValidationError::RouterIpOutOfBounds);
        }

        if self.server_keys.is_none() {
            return Err(RouterReleaseValidationError::MissingKeys);
        }

        Ok(self)
    }

    pub fn get_name(&self) -> String {
        format!("k8s-insider-router-{}", self.name)
    }

    pub fn get_namespace(&self) -> String {
        self.namespace.to_owned()
    }

    pub fn generate_router_metadata(&self) -> ObjectMeta {
        ObjectMeta {
            labels: Some(get_router_labels()),
            namespace: Some(self.get_namespace()),
            name: Some(self.get_name()),
            owner_references: Some(vec![self.owner.to_owned()]),
            ..Default::default()
        }
    }
}

impl From<NetworkService> for RouterReleaseService {
    fn from(value: NetworkService) -> Self {
        match value {
            NetworkService::ClusterIp { ip } => RouterReleaseService::ClusterIp { ip },
            NetworkService::NodePort {
                cluster_ip,
                predefined_ips,
            } => RouterReleaseService::NodePort {
                cluster_ip,
                predefined_ips,
            },
            NetworkService::LoadBalancer { cluster_ip } => {
                RouterReleaseService::LoadBalancer { cluster_ip }
            }
            NetworkService::ExternalIp {
                cluster_ip,
                ips: ip,
            } => RouterReleaseService::ExternalIp {
                cluster_ip,
                ips: ip,
            },
        }
    }
}
