use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{
            Capabilities, Container, EnvFromSource, PodSpec, PodTemplateSpec, Secret,
            SecretEnvSource, SecurityContext,
        },
    },
    apimachinery::pkg::apis::meta::v1::LabelSelector,
};
use kube::core::ObjectMeta;

use crate::resources::{labels::get_agent_labels, release::Release};

impl Release {
    pub fn generate_controller_deployment(&self, secret: &Secret) -> Deployment {
        let labels = get_agent_labels();
        let metadata = self.generate_tunnel_metadata();
        let metadata_name = metadata.name.as_ref().unwrap().to_owned();

        Deployment {
            metadata,
            spec: Some(DeploymentSpec {
                replicas: Some(1),
                selector: LabelSelector {
                    match_expressions: None,
                    match_labels: Some(labels.to_owned()),
                },
                template: PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some(labels.to_owned()),
                        ..Default::default()
                    }),
                    spec: Some(PodSpec {
                        // affinity: todo!(), // this should probably be introduced at some point
                        automount_service_account_token: Some(true),
                        containers: vec![Container {
                            env_from: Some(vec![EnvFromSource {
                                secret_ref: Some(SecretEnvSource {
                                    name: Some(secret.metadata.name.as_ref().unwrap().to_owned()),
                                    optional: Some(false),
                                }),
                                ..Default::default()
                            }]),
                            image: Some(self.agent_image_name.to_owned()),
                            image_pull_policy: Some("Always".to_owned()),
                            name: metadata_name,
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
                        ..Default::default()
                    }),
                },
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
