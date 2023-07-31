use std::net::IpAddr;

use derive_builder::Builder;
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::{core::ObjectMeta, Resource};
use thiserror::Error;

use crate::{
    helpers::AndIfSome,
    ip::{
        addrpair::{DualStackTryGet, IpAddrPair},
        netpair::IpNetPair,
        schema::IpNetFit,
        Contains,
    },
    resources::router::deployment::EXPOSED_PORT,
    wireguard::keys::{Keys, WgKey},
};

use self::secret::SERVER_PRIVATE_KEY_SECRET;

use super::{
    controller::ControllerRelease,
    crd::v1alpha1::network::{Network, NetworkService},
    labels::{get_network_manager_labels, get_router_labels},
    meta::NetworkMeta,
    ResourceGenerationError,
};

pub mod deployment;
pub mod rbac;
pub mod secret;
pub mod service;

#[derive(Debug, Builder)]
pub struct RouterRelease {
    pub controller_namespace: String,

    pub name: String,
    pub namespace: String,

    pub kube_dns: Option<IpAddrPair>,
    pub service_domain: Option<String>,
    pub service_cidr: IpNetPair,
    pub pod_cidr: IpNetPair,
    pub controller_image_name: String,
    pub network_manager_image_name: String,
    pub router_image_name: String,

    pub server_keys: Keys,
    pub peer_cidr: IpNetPair,
    pub router_ip: IpAddrPair,
    pub service: Option<RouterService>,

    pub owner: OwnerReference,
}

#[derive(Debug, Builder)]
pub struct RouterInfo {
    pub name: String,
    pub namespace: String,

    pub server_keys: Keys,
    pub peer_cidr: IpNetPair,
    pub router_ip: IpAddrPair,
    pub service: Option<RouterService>,

    pub owner: OwnerReference,
}

#[derive(Debug, Clone)]
pub enum RouterService {
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
        self.controller_namespace(controller_release.namespace.to_owned())
            .kube_dns(controller_release.kube_dns)
            .service_domain(controller_release.service_domain.to_owned())
            .service_cidr(controller_release.service_cidr)
            .pod_cidr(controller_release.pod_cidr)
            .controller_image_name(controller_release.controller_image_name.to_owned())
            .network_manager_image_name(controller_release.network_manager_image_name.to_owned())
            .router_image_name(controller_release.router_image_name.to_owned())
    }

    pub fn with_router_info(&mut self, router_info: RouterInfo) -> &mut Self {
        self.name(router_info.name)
            .namespace(router_info.namespace)
            .server_keys(router_info.server_keys)
            .peer_cidr(router_info.peer_cidr)
            .router_ip(router_info.router_ip)
            .service(router_info.service)
            .owner(router_info.owner)
    }
}

impl RouterInfoBuilder {
    pub fn with_network_crd(
        &mut self,
        crd: &Network,
    ) -> Result<&mut Self, ResourceGenerationError> {
        let server_public_key = crd
            .status
            .as_ref()
            .and_then(|status| status.server_public_key.as_ref())
            .map(|key| WgKey::from_base64(key))
            .transpose() // I love that I can do this
            .map_err(|_| {
                ResourceGenerationError::DependentInvalidData("server_public_key".into())
            })?;

        Ok(self
            .owner(crd.controller_owner_ref(&()).ok_or(
                ResourceGenerationError::DependentInvalidData("Network owner_ref".into()),
            )?)
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
                |builder, server_public_key| builder.server_keys(Keys::Public(server_public_key)),
            ))
    }

    pub fn with_private_key_from_env(&mut self) -> Result<&mut Self, ResourceGenerationError> {
        let env_var = std::env::var(SERVER_PRIVATE_KEY_SECRET).map_err(|_| {
            ResourceGenerationError::DependentMissingData("env.SERVER_PRIVATE_KEY".into())
        })?;
        let private_key = WgKey::from_base64(&env_var).map_err(|_| {
            ResourceGenerationError::DependentInvalidData("env.SERVER_PRIVATE_KEY".into())
        })?;
        let keys = Keys::from_private_key(private_key);

        Ok(self.server_keys(keys))
    }
}

impl RouterRelease {
    pub fn validated(self) -> Result<Self, RouterReleaseValidationError> {
        if !self.peer_cidr.contains(&self.router_ip) {
            return Err(RouterReleaseValidationError::RouterIpOutOfBounds);
        }

        Ok(self)
    }

    pub fn get_controller_namespace(&self) -> String {
        self.controller_namespace.to_owned()
    }

    pub fn get_allowed_cidrs(&self) -> Vec<IpNet> {
        self.pod_cidr
            .iter()
            .chain(self.service_cidr.iter())
            .chain(self.peer_cidr.iter())
            .collect()
    }

    pub fn get_allowed_fitcidrs(&self) -> Vec<IpNetFit> {
        self.pod_cidr
            .iter()
            .chain(self.service_cidr.iter())
            .chain(self.peer_cidr.iter())
            .map(|net| net.into())
            .collect()
    }

    pub fn generate_router_metadata(&self) -> ObjectMeta {
        ObjectMeta {
            labels: Some(get_router_labels(&self.name)),
            namespace: Some(self.get_router_namespace()),
            name: Some(self.get_router_name()),
            owner_references: Some(vec![self.owner.to_owned()]),
            ..Default::default()
        }
    }

    pub fn generate_network_manager_metadata(&self) -> ObjectMeta {
        ObjectMeta {
            labels: Some(get_network_manager_labels(&self.name)),
            namespace: Some(self.get_controller_namespace()),
            name: Some(self.get_network_manager_name()),
            owner_references: Some(vec![self.owner.to_owned()]),
            ..Default::default()
        }
    }
}

impl RouterInfo {
    pub fn generate_server_wg_config(&self) -> Result<String, ResourceGenerationError> {
        let address = self
            .router_ip
            .try_get_ipv4()
            .ok_or(ResourceGenerationError::MissingData(
                "router_ip.ipv4".into(),
            ))?;
        let private_key =
            self.server_keys
                .get_private_key()
                .ok_or(ResourceGenerationError::MissingData(
                    "server_keys.private_key".into(),
                ))?;
        let route_up = self.get_wgconfig_route_up();

        Ok(format!(
            "[Interface]
ListenPort = {EXPOSED_PORT}
Address = {address}
PrivateKey = {private_key}
PostUp = {route_up}"
        ))
    }

    fn get_wgconfig_route_up(&self) -> String {
        fn up4_snip(net: Ipv4Net) -> String {
            format!("ip -4 route add {} dev %i", net)
        }
    
        fn up6_snip(net: Ipv6Net) -> String {
            format!("ip -6 route add {} dev %i", net)
        }
    
        match self.peer_cidr {
            IpNetPair::Ipv4 { netv4 } => up4_snip(netv4),
            IpNetPair::Ipv6 { netv6 } => up6_snip(netv6),
            IpNetPair::Ipv4v6 { netv4, netv6 } => {
                format!("{} && {}", up4_snip(netv4), up6_snip(netv6))
            }
        }
    }
    
}

impl From<NetworkService> for RouterService {
    fn from(value: NetworkService) -> Self {
        match value {
            NetworkService::ClusterIp { ip } => RouterService::ClusterIp { ip },
            NetworkService::NodePort {
                cluster_ip,
                predefined_ips,
            } => RouterService::NodePort {
                cluster_ip,
                predefined_ips,
            },
            NetworkService::LoadBalancer { cluster_ip } => {
                RouterService::LoadBalancer { cluster_ip }
            }
            NetworkService::ExternalIp {
                cluster_ip,
                ips: ip,
            } => RouterService::ExternalIp {
                cluster_ip,
                ips: ip,
            },
        }
    }
}
