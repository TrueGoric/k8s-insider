use k8s_openapi::{
    api::core::v1::{Service, ServicePort, ServiceSpec},
    apimachinery::pkg::util::intstr::IntOrString,
};
use kube::core::ObjectMeta;

use crate::resources::{
    annotations::get_service_annotations,
    labels::get_release_labels,
    release::{Release, ReleaseService},
};

const PORT_NUMBER: i32 = 31313;

impl Release {
    pub fn generate_service(&self, port_name: &str) -> Option<Service> {
        if let ReleaseService::None = self.service {
            return None;
        }

        let labels = get_release_labels(&self.release_name);
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
        let annotations: Option<std::collections::BTreeMap<String, String>> =
            match &self.service {
                ReleaseService::NodePort { predefined_ips } => predefined_ips
                    .as_ref()
                    .and_then(|ips| Some(get_service_annotations(ips))),
                _ => None,
            };

        Some(Service {
            metadata: ObjectMeta {
                name: Some(self.release_name.to_owned()),
                namespace: Some(self.release_namespace.to_owned()),
                labels: Some(labels),
                annotations,
                ..Default::default()
            },
            spec,
            ..Default::default()
        })
    }
}
