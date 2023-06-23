use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::ConfigMap;

use super::ControllerRelease;

impl ControllerRelease {
    pub fn generate_configmap(&self) -> ConfigMap {
        let mut configmap_data = BTreeMap::from([
            (
                "KUBE_INSIDER_NAMESPACE".to_owned(),
                self.namespace.to_owned(),
            ),
            (
                "KUBE_INSIDER_SERVICE_CIDR".to_owned(),
                self.service_cidr.to_string(),
            ),
            (
                "KUBE_INSIDER_POD_CIDR".to_owned(),
                self.pod_cidr.to_string(),
            ),
            (
                "KUBE_INSIDER_AGENT_IMAGE_NAME".to_owned(),
                self.controller_image_name.to_string(),
            ),
            (
                "KUBE_INSIDER_TUNNEL_IMAGE_NAME".to_owned(),
                self.tunnel_image_name.to_string(),
            ),
        ]);

        if let Some(domain) = &self.service_domain {
            configmap_data.insert("KUBE_INSIDER_SERVICE_DOMAIN".to_owned(), domain.to_string());
        }

        if let Some(dns) = &self.kube_dns {
            configmap_data.insert("KUBE_INSIDER_DNS".to_owned(), dns.to_string());
        }

        ConfigMap {
            metadata: self.generate_default_metadata(),
            data: Some(configmap_data),
            ..Default::default()
        }
    }
}
