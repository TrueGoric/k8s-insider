use std::net::IpAddr;

use anyhow::anyhow;
use derive_builder::Builder;
use ipnet::IpNet;
use kube::core::ObjectMeta;

use super::labels::get_agent_labels;

#[derive(Debug, Builder)]
pub struct Release {
    pub namespace: String,
    pub agent_image_name: String,
    pub tunnel_image_name: String,
    pub server_private_key: String,
    pub kube_dns: Option<String>,
    pub service_domain: Option<String>,
    pub service_cidr: IpNet,
    pub pod_cidr: IpNet,
    pub peer_cidr: IpNet,
    pub router_ip: IpAddr,
    pub service: ReleaseService,
}

#[derive(Debug, Clone)]
pub enum ReleaseService {
    None,
    NodePort { predefined_ips: Option<String> },
    LoadBalancer,
    ExternalIp { ip: String },
}

impl Release {
    pub fn validated(self) -> anyhow::Result<Self> {
        if !self.peer_cidr.contains(&self.router_ip) {
            return Err(anyhow!("Router IP is not part of the peer network CIDR!"));
        }

        Ok(self)
    }

    pub fn generate_agent_metadata(&self) -> ObjectMeta {
        ObjectMeta {
            labels: Some(get_agent_labels()),
            namespace: Some(self.namespace.to_owned()),
            name: Some(format!("k8s-insider-agent")),
            ..Default::default()
        }
    }

    pub fn generate_clusterwide_agent_metadata(&self) -> ObjectMeta {
        ObjectMeta {
            labels: Some(get_agent_labels()),
            name: Some(format!("k8s-insider-agent")),
            ..Default::default()
        }
    }

    pub fn generate_tunnel_metadata(&self) -> ObjectMeta {
        ObjectMeta {
            labels: Some(get_agent_labels()),
            namespace: Some(self.namespace.to_owned()),
            name: Some(format!("k8s-insider-tunnel")),
            ..Default::default()
        }
    }
}
