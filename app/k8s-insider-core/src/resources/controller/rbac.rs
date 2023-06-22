use k8s_openapi::api::{
    core::v1::ServiceAccount,
    rbac::v1::{ClusterRole, ClusterRoleBinding, PolicyRule, Role, RoleBinding, RoleRef, Subject},
};
use kube::{CustomResourceExt, Resource};

use crate::resources::{crd::tunnel::Tunnel, release::Release};

impl Release {
    pub fn generate_controller_service_account(&self) -> ServiceAccount {
        ServiceAccount {
            metadata: self.generate_agent_metadata(),
            automount_service_account_token: Some(true),
            ..Default::default()
        }
    }

    pub fn generate_controller_cluster_role(&self) -> ClusterRole {
        ClusterRole {
            metadata: self.generate_clusterwide_agent_metadata(),
            rules: Some(vec![PolicyRule {
                api_groups: Some(vec!["".to_owned()]),
                resources: Some(vec!["nodes".to_owned()]),
                verbs: vec!["get".to_owned(), "watch".to_owned(), "list".to_owned()],
                ..Default::default()
            }]),
            ..Default::default()
        }
    }

    pub fn generate_controller_cluster_role_binding(
        &self,
        role: &Role,
        account: &ServiceAccount,
    ) -> ClusterRoleBinding {
        ClusterRoleBinding {
            metadata: self.generate_clusterwide_agent_metadata(),
            role_ref: RoleRef {
                kind: "Role".to_owned(),
                name: role.metadata.name.as_ref().unwrap().clone(),
                ..Default::default()
            },
            subjects: Some(vec![Subject {
                kind: "ServiceAccount".to_owned(),
                name: account.metadata.name.as_ref().unwrap().clone(),
                namespace: Some(account.metadata.namespace.as_ref().unwrap().clone()),
                ..Default::default()
            }]),
        }
    }

    pub fn generate_controller_role(&self) -> Role {
        let agent_metadata = self.generate_agent_metadata();
        let agent_metadata_name = agent_metadata.name.as_ref().unwrap().to_owned();
        let tunnel_metadata_name = self.generate_tunnel_metadata().name.unwrap();

        Role {
            metadata: agent_metadata,
            rules: Some(vec![
                PolicyRule {
                    api_groups: Some(vec!["".to_owned()]),
                    resources: Some(vec!["secrets".to_owned()]),
                    resource_names: Some(vec![agent_metadata_name.clone()]),
                    verbs: vec!["get".to_owned()],
                    ..Default::default()
                },
                PolicyRule {
                    api_groups: Some(vec!["".to_owned()]),
                    resources: Some(vec!["configmaps".to_owned()]),
                    resource_names: Some(vec![tunnel_metadata_name, agent_metadata_name]),
                    verbs: vec!["get".to_owned(), "create".to_owned(), "update".to_owned()],
                    ..Default::default()
                },
                PolicyRule {
                    api_groups: Some(vec![Tunnel::group(&()).into()]),
                    resources: Some(vec![Tunnel::crd_name().to_owned()]),
                    verbs: vec![
                        "get".to_owned(),
                        "watch".to_owned(),
                        "list".to_owned(),
                        "create".to_owned(),
                        "update".to_owned(),
                        "patch".to_owned(),
                        "delete".to_owned(),
                    ],
                    ..Default::default()
                },
            ]),
            ..Default::default()
        }
    }

    pub fn generate_controller_role_binding(&self, role: &Role, account: &ServiceAccount) -> RoleBinding {
        RoleBinding {
            metadata: self.generate_agent_metadata(),
            role_ref: RoleRef {
                kind: "Role".to_owned(),
                name: role.metadata.name.as_ref().unwrap().clone(),
                ..Default::default()
            },
            subjects: Some(vec![Subject {
                kind: "ServiceAccount".to_owned(),
                name: account.metadata.name.as_ref().unwrap().clone(),
                namespace: Some(account.metadata.namespace.as_ref().unwrap().clone()),
                ..Default::default()
            }]),
        }
    }
}
