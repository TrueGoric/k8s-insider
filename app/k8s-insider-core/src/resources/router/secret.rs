use std::collections::BTreeMap;

use k8s_openapi::{api::core::v1::Secret, ByteString};

use crate::resources::ResourceGenerationError;

use super::RouterRelease;

pub const SERVER_PRIVATE_KEY_SECRET: &str = "SERVER_PRIVATE_KEY";

impl RouterRelease {
    pub fn generate_secret(&self) -> Result<Secret, ResourceGenerationError> {
        let secret_data = BTreeMap::from([(
            SERVER_PRIVATE_KEY_SECRET.to_owned(),
            ByteString(
                self.server_keys
                    .get_private_key()
                    .ok_or(ResourceGenerationError::MissingData(
                        "server_keys.private_key".into(),
                    ))?
                    .to_base64()
                    .into_bytes(),
            ),
        )]);

        Ok(Secret {
            metadata: self.generate_router_metadata(),
            data: Some(secret_data),
            ..Default::default()
        })
    }
}
