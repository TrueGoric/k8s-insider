use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::ConfigMap;
use kube::core::ObjectMeta;

use super::{release::Release, get_release_labels};

pub fn generate_configmap(release_info: &Release) -> ConfigMap {
    let mut configmap_data = BTreeMap::from([
        ("SERVER_ADDRESS".to_owned(), "0.0.0.0".to_owned()),
        ("SERVER_PORT".to_owned(), "31111".to_owned()),
        ("KUBE_SERVICE_CIDR".to_owned(), release_info.service_cidr.to_owned()),
        ("KUBE_POD_CIDR".to_owned(), release_info.pod_cidr.to_owned()),
    ]);

    if let Some(dns) = &release_info.kube_dns {
        configmap_data.insert("PEER_DNS".to_owned(), dns.to_owned());
    }

    let configmap = ConfigMap {
        metadata: ObjectMeta {
            labels: Some(get_release_labels(&release_info.release_name)),
            name: Some(release_info.release_name.to_owned()),
            namespace: Some(release_info.release_namespace.to_owned()),
            ..Default::default()
        },
        data: Some(configmap_data),
        ..Default::default()
    };

    configmap
}