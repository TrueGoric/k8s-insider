use k8s_openapi::{
    api::{
        apps::v1::Deployment,
        core::v1::{Service, ServicePort, ServiceSpec},
    },
    apimachinery::pkg::util::intstr::IntOrString,
};
use kube::core::ObjectMeta;

use crate::resources::{
    annotations::get_service_annotations,
    labels::get_tunnel_labels,
    release::{Release, ReleaseService},
};

const PORT_NUMBER: i32 = 31313;

impl Release {
    pub fn generate_tunnel_service(&self, deployment: &Deployment) -> Option<Service> {
        if let ReleaseService::None = self.service {
            return None;
        }

        let port_name = extract_port_name(deployment);
        let labels = get_tunnel_labels();
        let port = ServicePort {
            name: Some(port_name.to_owned()),
            port: PORT_NUMBER,
            protocol: Some("UDP".to_owned()),
            target_port: Some(IntOrString::String(port_name.to_owned())),
            ..Default::default()
        };
        let spec = match &self.service {
            ReleaseService::None => None,
            ReleaseService::NodePort { .. } => Some(ServiceSpec {
                ports: Some(vec![port]),
                selector: Some(labels.to_owned()),
                type_: Some("NodePort".to_owned()),
                ..Default::default()
            }),
            ReleaseService::LoadBalancer => Some(ServiceSpec {
                ports: Some(vec![port]),
                selector: Some(labels.to_owned()),
                type_: Some("LoadBalancer".to_owned()),
                ..Default::default()
            }),
            ReleaseService::ExternalIp { ip } => todo!(),
        };

        let annotations: Option<std::collections::BTreeMap<String, String>> = match &self.service {
            ReleaseService::NodePort { predefined_ips } => predefined_ips
                .as_ref()
                .and_then(|ips| Some(get_service_annotations(ips))),
            _ => None,
        };

        let metadata = ObjectMeta {
            annotations,
            ..self.generate_tunnel_metadata()
        };

        Some(Service {
            metadata,
            spec,
            ..Default::default()
        })
    }
}

fn extract_port_name(deployment: &Deployment) -> &str {
    deployment
        .spec
        .as_ref()
        .unwrap()
        .template
        .spec
        .as_ref()
        .unwrap()
        .containers
        .first()
        .unwrap()
        .ports
        .as_ref()
        .unwrap()
        .first()
        .unwrap()
        .name
        .as_ref()
        .unwrap() // ┌(˘⌣˘)ʃ
}
