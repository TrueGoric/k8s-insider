use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{
            Capabilities, Container, ContainerPort, EnvFromSource, PodSpec, PodTemplateSpec,
            Secret, SecretEnvSource, SecurityContext, ServiceAccount,
        },
    },
    apimachinery::pkg::apis::meta::v1::LabelSelector,
};
use kube::core::ObjectMeta;

use crate::resources::{labels::get_router_labels, ResourceGenerationError};

use super::RouterRelease;

pub const EXPOSED_PORT: i32 = 55555;
pub const EXPOSED_PORT_NAME: &str = "vpn";
pub const EXPOSED_PORT_PROTOCOL: &str = "UDP";

impl RouterRelease {
    pub fn generate_deployment(
        &self,
        secret: &Secret,
        service_account: &ServiceAccount,
    ) -> Result<Deployment, ResourceGenerationError> {
        let labels = get_router_labels();
        let metadata = self.generate_router_metadata();
        let metadata_name = metadata
            .name
            .as_ref()
            .ok_or(ResourceGenerationError::DependentMissingMetadataName)?
            .to_owned();
        let secret_name = secret
            .metadata
            .name
            .as_ref()
            .ok_or(ResourceGenerationError::DependentMissingMetadataName)?
            .to_owned();
        let service_account_name = service_account
            .metadata
            .name
            .as_ref()
            .ok_or(ResourceGenerationError::DependentMissingMetadataName)?
            .to_owned();

        let pod_spec = PodSpec {
            // affinity: todo!(), // this should probably be introduced at some point
            automount_service_account_token: Some(true),
            containers: vec![Container {
                env_from: Some(vec![EnvFromSource {
                    secret_ref: Some(SecretEnvSource {
                        name: Some(secret_name),
                        optional: Some(false),
                    }),
                    ..Default::default()
                }]),
                image: Some(self.tunnel_image_name.to_owned()),
                image_pull_policy: Some("Always".to_owned()),
                name: metadata_name,
                ports: Some(vec![ContainerPort {
                    name: Some(EXPOSED_PORT_NAME.to_owned()),
                    container_port: EXPOSED_PORT,
                    protocol: Some(EXPOSED_PORT_PROTOCOL.to_owned()),
                    ..Default::default()
                }]),
                // resources: todo!(), // this too
                security_context: Some(SecurityContext {
                    allow_privilege_escalation: Some(true),
                    capabilities: Some(Capabilities {
                        add: Some(vec!["NET_ADMIN".to_owned()]),
                        ..Default::default()
                    }),
                    privileged: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            service_account_name: Some(service_account_name),
            ..Default::default()
        };

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
                    spec: Some(pod_spec),
                },
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
