use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{
            Capabilities, ConfigMapEnvSource, Container, ContainerPort, EnvFromSource, PodSpec,
            PodTemplateSpec, SecurityContext,
        },
    },
    apimachinery::pkg::apis::meta::v1::LabelSelector,
};
use kube::core::ObjectMeta;

use crate::resources::labels::get_release_labels;

use super::release::Release;

const EXPOSED_PORT: i32 = 55555;
const EXPOSED_PORT_NAME: &str = "vpn";
const EXPOSED_PORT_PROTOCOL: &str = "UDP";

pub fn generate_deployment(release_info: &Release, configmap_name: &str) -> Deployment {
    let labels = get_release_labels(&release_info.release_name);

    Deployment {
        metadata: ObjectMeta {
            name: Some(release_info.release_name.to_owned()),
            namespace: Some(release_info.release_namespace.to_owned()),
            labels: Some(labels.to_owned()),
            ..Default::default()
        },
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
                                name: Some(configmap_name.to_owned()),
                                optional: Some(false),
                            }),
                            ..Default::default()
                        }]),
                        image: Some(release_info.image_name.to_owned()),
                        image_pull_policy: Some("Always".to_owned()),
                        name: release_info.release_name.to_owned(),
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
