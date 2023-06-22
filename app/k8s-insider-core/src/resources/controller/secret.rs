use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::Secret;

use crate::resources::release::Release;

impl Release {
    pub fn generate_controller_secret(&self) -> Secret {
        let data = BTreeMap::from([(
            "SERVER_PRIVATE_KEY".to_owned(),
            self.server_private_key.to_owned(),
        )]);

        let secret = Secret {
            metadata: self.generate_agent_metadata(),
            string_data: Some(data),
            ..Default::default()
        };

        secret
    }
}
