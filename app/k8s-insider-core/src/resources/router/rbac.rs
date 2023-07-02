use k8s_openapi::api::{
    core::v1::ServiceAccount,
    rbac::v1::{RoleBinding, RoleRef, Subject},
};

use crate::{
    helpers::RequireMetadata, resources::ResourceGenerationError, NETWORK_MANAGER_CLUSTERROLE_NAME,
    ROUTER_CLUSTERROLE_NAME,
};

use super::RouterRelease;

impl RouterRelease {
    pub fn generate_router_service_account(&self) -> ServiceAccount {
        ServiceAccount {
            metadata: self.generate_router_metadata(),
            automount_service_account_token: Some(true),
            ..Default::default()
        }
    }

    pub fn generate_network_manager_service_account(&self) -> ServiceAccount {
        ServiceAccount {
            metadata: self.generate_network_manager_metadata(),
            automount_service_account_token: Some(true),
            ..Default::default()
        }
    }

    pub fn generate_router_role_binding(
        &self,
        account: &ServiceAccount,
    ) -> Result<RoleBinding, ResourceGenerationError> {
        Ok(RoleBinding {
            metadata: self.generate_router_metadata(),
            role_ref: RoleRef {
                kind: "ClusterRole".to_owned(),
                name: ROUTER_CLUSTERROLE_NAME.to_owned(),
                ..Default::default()
            },
            subjects: Some(vec![Subject {
                kind: "ServiceAccount".to_owned(),
                name: account
                    .require_name_or(ResourceGenerationError::DependentMissingMetadataName)?
                    .to_owned(),
                namespace: Some(
                    account
                        .require_namespace_or(
                            ResourceGenerationError::DependentMissingMetadataName,
                        )?
                        .to_owned(),
                ),
                ..Default::default()
            }]),
        })
    }

    pub fn generate_network_manager_role_binding(
        &self,
        account: &ServiceAccount,
    ) -> Result<RoleBinding, ResourceGenerationError> {
        Ok(RoleBinding {
            metadata: self.generate_network_manager_metadata(),
            role_ref: RoleRef {
                kind: "ClusterRole".to_owned(),
                name: NETWORK_MANAGER_CLUSTERROLE_NAME.to_owned(),
                ..Default::default()
            },
            subjects: Some(vec![Subject {
                kind: "ServiceAccount".to_owned(),
                name: account
                    .require_name_or(ResourceGenerationError::DependentMissingMetadataName)?
                    .to_owned(),
                namespace: Some(
                    account
                        .require_namespace_or(
                            ResourceGenerationError::DependentMissingMetadataName,
                        )?
                        .to_owned(),
                ),
                ..Default::default()
            }]),
        })
    }
}
