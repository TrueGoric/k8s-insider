use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::ConfigMap;

use crate::resources::{release::Release, templates::server_interface_template};

use super::deployment::EXPOSED_PORT;

impl Release {
    pub fn generate_tunnel_configmap(&self) -> ConfigMap {
        let configmap_data = BTreeMap::from([(
            "wg0.conf".to_owned(),
            server_interface_template(
                &self.router_ip,
                EXPOSED_PORT as u32,
                &self.server_private_key,
            ),
        )]);

        let configmap = ConfigMap {
            metadata: self.generate_tunnel_metadata(),
            data: Some(configmap_data),
            ..Default::default()
        };

        configmap
    }
}
