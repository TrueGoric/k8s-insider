use std::net::IpAddr;
use anyhow::anyhow;
use derive_builder::Builder;
use kube::{api::PatchParams, core::ObjectMeta, Client};
use log::debug;

use crate::{kubernetes::operations::{create_namespace_if_not_exists, create_resource}, FIELD_MANAGER, ippair::{IpAddrPair, IpNetPair, Contains}};

use super::{controller::ControllerRelease, labels::get_router_labels};

pub mod configmap;
pub mod deployment;
pub mod rbac;
pub mod service;

#[derive(Debug, Builder)]
pub struct RouterRelease {
    pub namespace: String,
    pub kube_dns: Option<IpAddrPair>,
    pub service_domain: Option<String>,
    pub service_cidr: IpNetPair,
    pub pod_cidr: IpNetPair,
    pub agent_image_name: String,
    pub tunnel_image_name: String,

    pub server_private_key: String,
    pub peer_cidr: IpNetPair,
    pub router_ip: IpAddrPair,
    pub service: RouterReleaseService,
}

#[derive(Debug, Clone)]
pub enum RouterReleaseService {
    None,
    NodePort { predefined_ips: Option<Vec<IpAddr>> },
    LoadBalancer,
    ExternalIp { ip: String },
}

impl RouterReleaseBuilder {
    pub fn with_controller(&mut self, controller_release: ControllerRelease) -> &mut Self {
        self.namespace(controller_release.namespace)
            .kube_dns(controller_release.kube_dns)
            .service_domain(controller_release.service_domain)
            .service_cidr(controller_release.service_cidr)
            .pod_cidr(controller_release.pod_cidr)
            .agent_image_name(controller_release.controller_image_name)
            .tunnel_image_name(controller_release.tunnel_image_name)
    }

    // pub fn with_network_crd(&mut self, crd: &Network) -> &mut Self {
    //     self.peer_cidr(crd.spec.peer_cidr.to_owned())
    // }
}

impl RouterRelease {
    pub fn validated(self) -> anyhow::Result<Self> {
        if !self.peer_cidr.contains(&self.router_ip) {
            return Err(anyhow!("Router IP is not part of the peer network CIDR!"));
        }

        Ok(self)
    }

    pub fn generate_router_metadata(&self) -> ObjectMeta {
        ObjectMeta {
            labels: Some(get_router_labels()),
            namespace: Some(self.namespace.to_owned()),
            name: Some("k8s-insider-tunnel".to_string()),
            ..Default::default()
        }
    }

    pub async fn deploy(&self, client: &Client, dry_run: bool) -> anyhow::Result<()> {
        let namespace = &self.namespace;
        let configmap = self.generate_configmap();
        let deployment = self.generate_deployment(&configmap);
        let service = self.generate_service(&deployment);

        debug!("{configmap:#?}");
        debug!("{deployment:#?}");
        debug!("{service:#?}");

        let patch_params = PatchParams {
            dry_run,
            field_manager: Some(FIELD_MANAGER.to_owned()),
            ..Default::default()
        };

        create_namespace_if_not_exists(client, &patch_params, namespace).await?;
        create_resource(client, &deployment, &patch_params).await?;
        create_resource(client, &configmap, &patch_params).await?;

        if let Some(service) = service {
            create_resource(client, &service, &patch_params).await?;
        }

        Ok(())
    }
}
