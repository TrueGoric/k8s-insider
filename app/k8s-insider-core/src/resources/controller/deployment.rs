use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{
            ConfigMap, ConfigMapEnvSource, Container, EnvFromSource, PodSpec, PodTemplateSpec,
            ServiceAccount,
        },
    },
    apimachinery::pkg::apis::meta::v1::LabelSelector,
};
use kube::core::ObjectMeta;

use crate::{
    helpers::RequireMetadata,
    resources::{labels::get_controller_labels, ResourceGenerationError},
};

use super::ControllerRelease;

impl ControllerRelease {
    pub fn generate_deployment(
        &self,
        configmap: &ConfigMap,
        service_account: &ServiceAccount,
    ) -> Result<Deployment, ResourceGenerationError> {
        let labels = get_controller_labels();
        let metadata = self.generate_default_metadata();
        let metadata_name = metadata
            .name
            .as_ref()
            .ok_or(ResourceGenerationError::DependentMissingMetadataName)?
            .to_owned();
        let configmap_name = configmap
            .require_name_or(ResourceGenerationError::DependentMissingMetadataName)?
            .to_owned();
        let service_account_name = service_account
            .require_name_or(ResourceGenerationError::DependentMissingMetadataName)?
            .to_owned();

        Ok(Deployment {
            metadata,
            spec: Some(DeploymentSpec {
                replicas: Some(1),
                selector: LabelSelector {
                    match_expressions: None,
                    match_labels: Some(labels.to_owned()),
                },
                template: PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some(labels),
                        ..Default::default()
                    }),
                    spec: Some(PodSpec {
                        // affinity: todo!(), // this should probably be introduced at some point
                        automount_service_account_token: Some(true),
                        containers: vec![Container {
                            env_from: Some(vec![EnvFromSource {
                                config_map_ref: Some(ConfigMapEnvSource {
                                    name: Some(configmap_name),
                                    optional: Some(false),
                                }),
                                ..Default::default()
                            }]),
                            image: Some(self.controller_image_name.to_owned()),
                            image_pull_policy: Some("IfNotPresent".to_owned()),
                            name: metadata_name,
                            // resources: todo!(), // this too
                            ..Default::default()
                        }],
                        service_account_name: Some(service_account_name),
                        ..Default::default()
                    }),
                },
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
