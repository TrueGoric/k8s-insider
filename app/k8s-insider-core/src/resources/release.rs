use derive_builder::Builder;
use kube::core::ObjectMeta;

use super::labels::{get_common_labels, get_release_labels};

#[derive(Debug, Builder)]
pub struct Release {
    pub release_name: String,
    pub release_namespace: String,
    pub image_name: String,
    pub server_private_key: String,
    pub kube_dns: Option<String>,
    pub service_cidr: String,
    pub service_domain: Option<String>,
    pub pod_cidr: String,
    pub service: ReleaseService
}

#[derive(Debug, Clone)]
pub enum ReleaseService {
    None,
    NodePort {
        predefined_ips: Option<String>
    },
    LoadBalancer,
    ExternalIp {
        ip: String
    }
}

impl Release {
    pub fn generate_metadata(&self, name_suffix: &str) -> ObjectMeta {
        ObjectMeta {
            namespace: Some(self.release_namespace.to_owned()),
            ..self.generate_clusterwide_metadata(name_suffix)
        }
    }

    pub fn generate_clusterwide_metadata(&self, name_suffix: &str) -> ObjectMeta {
        ObjectMeta {
            labels: Some(get_release_labels(&self.release_name)),
            name: Some(format!("{}-{}", self.release_name, name_suffix)),
            ..Default::default()
        }
    }

    pub fn generate_agent_metadata(&self) -> ObjectMeta {
        self.generate_metadata("agent")
    }

    pub fn generate_clusterwide_agent_metadata(&self) -> ObjectMeta {
        self.generate_clusterwide_metadata("tunnel")
    }

    pub fn generate_tunnel_metadata(&self) -> ObjectMeta {
        self.generate_metadata("tunnel")
    }

}