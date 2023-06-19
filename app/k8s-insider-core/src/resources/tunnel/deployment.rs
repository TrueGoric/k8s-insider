use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{
            Capabilities, ConfigMap, ConfigMapEnvSource, Container, ContainerPort, EnvFromSource,
            PodSpec, PodTemplateSpec, SecurityContext,
        },
    },
    apimachinery::pkg::apis::meta::v1::LabelSelector,
};
use kube::core::ObjectMeta;

use crate::resources::{labels::get_tunnel_labels, release::Release};

const EXPOSED_PORT: i32 = 55555;
const EXPOSED_PORT_NAME: &str = "vpn";
const EXPOSED_PORT_PROTOCOL: &str = "UDP";

impl Release {
    pub fn generate_tunnel_deployment(&self, configmap: &ConfigMap) -> Deployment {
        let labels = get_tunnel_labels();
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
                        automount_service_account_token: Some(false),
                        containers: vec![Container {
                            env_from: Some(vec![EnvFromSource {
                                config_map_ref: Some(ConfigMapEnvSource {
                                    name: Some(
                                        configmap.metadata.name.as_ref().unwrap().to_owned(),
                                    ),
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
                        ..Default::default()
                    }),
                },
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
