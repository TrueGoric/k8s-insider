use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{
            Capabilities, ConfigMapEnvSource, Container, ContainerPort, EnvFromSource, PodSpec,
            PodTemplateSpec, Secret, SecretEnvSource, SecurityContext, ServiceAccount, EnvVar,
        },
    },
    apimachinery::pkg::apis::meta::v1::LabelSelector,
};
use kube::core::ObjectMeta;

use crate::{
    helpers::RequireMetadata,
    resources::{
        controller::CONTROLLER_RELEASE_NAME,
        labels::{get_network_manager_labels, get_router_labels},
        ResourceGenerationError,
    },
};

use super::RouterRelease;

pub const EXPOSED_PORT: i32 = 55555;
pub const EXPOSED_PORT_NAME: &str = "vpn";
pub const EXPOSED_PORT_PROTOCOL: &str = "UDP";

impl RouterRelease {
    pub fn generate_router_deployment(
        &self,
        secret: &Secret,
        service_account: &ServiceAccount,
    ) -> Result<Deployment, ResourceGenerationError> {
        let labels = get_router_labels(&self.name);
        let metadata = self.generate_router_metadata();
        let metadata_name = metadata
            .name
            .as_ref()
            .ok_or(ResourceGenerationError::DependentMissingMetadataName)?
            .to_owned();
        let secret_name = secret
            .require_name_or(ResourceGenerationError::DependentMissingMetadataName)?
            .to_owned();
        let service_account_name = service_account
            .require_name_or(ResourceGenerationError::DependentMissingMetadataName)?
            .to_owned();

        let pod_spec = PodSpec {
            // affinity: todo!(), // this should probably be introduced at some point
            automount_service_account_token: Some(true),
            containers: vec![Container {
                env: Some(vec![EnvVar {
                    name: "KUBE_INSIDER_NETWORK_NAME".to_owned(),
                    value: Some(self.name.to_owned()),
                    ..Default::default()
                },
                EnvVar {
                    name: "KUBE_INSIDER_NETWORK_NAMESPACE".to_owned(),
                    value: Some(self.namespace.to_owned()),
                    ..Default::default()
                }]),
                env_from: Some(vec![EnvFromSource {
                    secret_ref: Some(SecretEnvSource {
                        name: Some(secret_name),
                        optional: Some(false),
                    }),
                    ..Default::default()
                }]),
                image: Some(self.router_image.to_owned()),
                image_pull_policy: Some("IfNotPresent".to_owned()),
                name: metadata_name,
                ports: Some(vec![ContainerPort {
                    name: Some(EXPOSED_PORT_NAME.to_owned()),
                    container_port: EXPOSED_PORT,
                    protocol: Some(EXPOSED_PORT_PROTOCOL.to_owned()),
                    ..Default::default()
                }]),
                // resources: todo!(), // this too
                security_context: Some(SecurityContext {
                    allow_privilege_escalation: Some(false),
                    capabilities: Some(Capabilities {
                        add: Some(vec!["NET_ADMIN".to_owned()]),
                        ..Default::default()
                    }),
                    privileged: Some(false),
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

    pub fn generate_network_manager_deployment(
        &self,
        service_account: &ServiceAccount,
    ) -> Result<Deployment, ResourceGenerationError> {
        let labels = get_network_manager_labels(&self.name);
        let metadata = self.generate_network_manager_metadata();
        let metadata_name = metadata
            .name
            .as_ref()
            .ok_or(ResourceGenerationError::DependentMissingMetadataName)?
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
                                    name: Some(CONTROLLER_RELEASE_NAME.to_owned()),
                                    optional: Some(false),
                                }),
                                ..Default::default()
                            }]),
                            env: Some(vec![EnvVar {
                                name: "KUBE_INSIDER_NETWORK_NAME".to_owned(),
                                value: Some(self.name.to_owned()),
                                ..Default::default()
                            },
                            EnvVar {
                                name: "KUBE_INSIDER_NETWORK_NAMESPACE".to_owned(),
                                value: Some(self.namespace.to_owned()),
                                ..Default::default()
                            }]),
                            image: Some(self.network_manager_image.to_owned()),
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
