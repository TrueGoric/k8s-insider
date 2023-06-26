use std::collections::BTreeMap;

use k8s_openapi::{
    api::{
        apps::v1::Deployment,
        core::v1::{Service, ServicePort, ServiceSpec},
    },
    apimachinery::pkg::util::intstr::IntOrString,
};
use kube::core::ObjectMeta;

use crate::{
    ippair::IpAddrPair,
    resources::{annotations::get_service_annotations, labels::get_router_labels},
};

use super::{RouterRelease, RouterReleaseService};

const PORT_NUMBER: i32 = 31313;

impl RouterRelease {
    pub fn generate_service(&self, deployment: &Deployment) -> Option<Service> {
        let service = self.service.as_ref()?;
        let port_name = extract_port_name(deployment);
        let labels = get_router_labels();
        let port = ServicePort {
            name: Some(port_name.to_owned()),
            port: PORT_NUMBER,
            protocol: Some("UDP".to_owned()),
            target_port: Some(IntOrString::String(port_name.to_owned())),
            ..Default::default()
        };
        let spec = match service {
            RouterReleaseService::ClusterIp { ip: cluster_ip } => Some(get_base_servicespec(
                "ClusterIP",
                Some(labels),
                cluster_ip,
                port,
            )),
            RouterReleaseService::NodePort {
                cluster_ip,
                predefined_ips: _,
            } => Some(get_base_servicespec(
                "NodePort",
                Some(labels),
                cluster_ip,
                port,
            )),
            RouterReleaseService::LoadBalancer { cluster_ip } => Some(get_base_servicespec(
                "LoadBalancer",
                Some(labels),
                cluster_ip,
                port,
            )),
            RouterReleaseService::ExternalIp {
                cluster_ip,
                ips: ip,
            } => Some(ServiceSpec {
                external_ips: Some(ip.iter().map(|ip| ip.to_string()).collect()),
                ..get_base_servicespec("ClusterIp", Some(labels), cluster_ip, port)
            }),
        };

        let annotations: Option<std::collections::BTreeMap<String, String>> = match service {
            RouterReleaseService::NodePort { predefined_ips, .. } => predefined_ips
                .as_ref()
                .map(|ips| get_service_annotations(ips)),
            _ => None,
        };

        let metadata = ObjectMeta {
            annotations,
            ..self.generate_router_metadata()
        };

        Some(Service {
            metadata,
            spec,
            ..Default::default()
        })
    }
}

fn get_base_servicespec(
    type_: &str,
    selector: Option<BTreeMap<String, String>>,
    cluster_ip: &Option<IpAddrPair>,
    port: ServicePort,
) -> ServiceSpec {
    ServiceSpec {
        selector,
        type_: Some(type_.to_owned()),
        ports: Some(vec![port]),
        cluster_ips: cluster_ip.map(|ip| ip.into()),
        ip_family_policy: Some("PreferDualStack".to_owned()),
        ..Default::default()
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
