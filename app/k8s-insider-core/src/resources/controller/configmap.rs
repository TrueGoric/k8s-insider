use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::ConfigMap;

use crate::resources::release::Release;

impl Release {
    pub fn generate_controller_configmap(&self) -> ConfigMap {
        let mut configmap_data = BTreeMap::from([
            ("SERVER_ADDRESS".to_owned(), "0.0.0.0".to_owned()),
            ("SERVER_PORT".to_owned(), "31111".to_owned()),
            ("KUBE_SERVICE_CIDR".to_owned(), self.service_cidr.to_string()),
            ("KUBE_POD_CIDR".to_owned(), self.pod_cidr.to_string()),
        ]);

        if let Some(dns) = &self.kube_dns {
            configmap_data.insert("PEER_DNS".to_owned(), dns.to_owned());
        }

        let configmap = ConfigMap {
            metadata: self.generate_agent_metadata(),
            data: Some(configmap_data),
            ..Default::default()
        };

        configmap
    }
}
