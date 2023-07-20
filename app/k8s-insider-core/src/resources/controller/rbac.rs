use k8s_openapi::api::{
    core::v1::ServiceAccount,
    rbac::v1::{ClusterRole, ClusterRoleBinding, PolicyRule, RoleRef, Subject},
};
use kube::Resource;

use crate::{
    resources::{
        crd::v1alpha1::{network::Network, tunnel::Tunnel},
        ResourceGenerationError,
    },
    CONTROLLER_CLUSTERROLE_NAME, NETWORK_MANAGER_CLUSTERROLE_NAME, ROUTER_CLUSTERROLE_NAME,
};

use super::ControllerRelease;

impl ControllerRelease {
    pub fn generate_controller_service_account(&self) -> ServiceAccount {
        ServiceAccount {
            metadata: self.generate_default_metadata(),
            automount_service_account_token: Some(true),
            ..Default::default()
        }
    }

    pub fn generate_controller_clusterrole(&self) -> ClusterRole {
        // RATIONALE: read nodes to generate NodePort service addresses
        let read_nodes = PolicyRule {
            api_groups: Some(vec!["".to_owned()]),
            resources: Some(vec!["nodes".to_owned()]),
            verbs: vec!["get".to_owned(), "watch".to_owned(), "list".to_owned()],
            ..Default::default()
        };

        // RATIONALE: bind 'k8s-insider-router' and 'k8s-insider-network-manager' roles
        //            to routers and network-managers to allow them to manage VPN-network-specific resources
        let bind_router_cluster_role = PolicyRule {
            api_groups: Some(vec!["rbac.authorization.k8s.io".to_owned()]),
            resources: Some(vec!["clusterroles".to_owned()]),
            resource_names: Some(vec![
                ROUTER_CLUSTERROLE_NAME.to_owned(),
                NETWORK_MANAGER_CLUSTERROLE_NAME.to_owned(),
            ]),
            verbs: vec!["bind".to_owned()],
            ..Default::default()
        };

        // RATIONALE: create/patch serviceaccounts to create router accounts,
        //            watch/list to watch serviceaccounts owned by networks/routers
        let create_list_serviceaccounts = PolicyRule {
            api_groups: Some(vec!["".to_owned()]),
            resources: Some(vec!["serviceaccounts".to_owned()]),
            verbs: vec![
                "create".to_owned(),
                "patch".to_owned(),
                "watch".to_owned(),
                "list".to_owned(),
            ],
            ..Default::default()
        };

        // RATIONALE: create/patch rolebindings to attach 'k8s-insider-router' and 'k8s-insider-network-manager'
        //            roles to router accounts, watch/list to watch rolebindings owned by networks/routers
        let create_list_rolebindings = PolicyRule {
            api_groups: Some(vec!["rbac.authorization.k8s.io".to_owned()]),
            resources: Some(vec!["rolebindings".to_owned()]),
            verbs: vec![
                "create".to_owned(),
                "patch".to_owned(),
                "watch".to_owned(),
                "list".to_owned(),
            ],
            ..Default::default()
        };

        // RATIONALE: create/patch secrets for networks,
        //            watch/list to watch secrets owned by networks/routers
        let create_list_secrets = PolicyRule {
            api_groups: Some(vec!["".to_owned()]),
            resources: Some(vec!["secrets".to_owned()]),
            verbs: vec![
                "create".to_owned(),
                "patch".to_owned(),
                "get".to_owned(),
                "watch".to_owned(),
                "list".to_owned(),
            ],
            ..Default::default()
        };

        // RATIONALE: create/patch services for networks,
        //            watch/list to watch for network changes,
        //            get to acquire info for tunnels
        let create_read_services = PolicyRule {
            api_groups: Some(vec!["".to_owned()]),
            resources: Some(vec!["services".to_owned()]),
            verbs: vec![
                "create".to_owned(),
                "patch".to_owned(),
                "get".to_owned(),
                "watch".to_owned(),
                "list".to_owned(),
            ],
            ..Default::default()
        };

        // RATIONALE: manage deployments to materialize and handle the lifecycle of a router
        let manage_deployments = PolicyRule {
            api_groups: Some(vec!["apps".to_owned()]),
            resources: Some(vec!["deployments".to_owned()]),
            verbs: vec![
                "create".to_owned(),
                "update".to_owned(),
                "patch".to_owned(),
                "get".to_owned(),
                "watch".to_owned(),
                "list".to_owned(),
            ],
            ..Default::default()
        };

        // RATIONALE: read networks to create network related resources (pods, secrets),
        //            update networks to put in status updates and managed info (public_keys, etc.)
        let manage_networks = PolicyRule {
            api_groups: Some(vec![Network::group(&()).into()]),
            resources: Some(vec![Network::plural(&()).into()]),
            verbs: vec!["get".to_owned(), "watch".to_owned(), "list".to_owned()],
            ..Default::default()
        };

        // RATIONALE: update network statuses to put in updates and managed info (public_keys, etc.)
        let update_network_statuses: PolicyRule = PolicyRule {
            api_groups: Some(vec![Network::group(&()).into()]),
            resources: Some(vec![format!("{}/status", Network::plural(&()))]),
            verbs: vec!["update".to_owned(), "patch".to_owned()],
            ..Default::default()
        };

        ClusterRole {
            metadata: self.generate_clusterwide_metadata(CONTROLLER_CLUSTERROLE_NAME),
            rules: Some(vec![
                read_nodes,
                bind_router_cluster_role,
                create_list_serviceaccounts,
                create_list_rolebindings,
                create_list_secrets,
                create_read_services,
                manage_deployments,
                manage_networks,
                update_network_statuses,
            ]),
            ..Default::default()
        }
    }

    pub fn generate_controller_cluster_role_binding(
        &self,
        role: &ClusterRole,
        account: &ServiceAccount,
    ) -> Result<ClusterRoleBinding, ResourceGenerationError> {
        Ok(ClusterRoleBinding {
            metadata: self.generate_clusterwide_default_metadata(),
            role_ref: RoleRef {
                kind: "ClusterRole".to_owned(),
                name: role
                    .metadata
                    .name
                    .as_ref()
                    .ok_or(ResourceGenerationError::DependentMissingMetadataName)?
                    .clone(),
                ..Default::default()
            },
            subjects: Some(vec![Subject {
                kind: "ServiceAccount".to_owned(),
                name: account
                    .metadata
                    .name
                    .as_ref()
                    .ok_or(ResourceGenerationError::DependentMissingMetadataName)?
                    .clone(),
                namespace: Some(
                    account
                        .metadata
                        .namespace
                        .as_ref()
                        .ok_or(ResourceGenerationError::DependentMissingMetadataNamespace)?
                        .clone(),
                ),
                ..Default::default()
            }]),
        })
    }

    pub fn generate_network_manager_clusterrole(&self) -> ClusterRole {
        // RATIONALE: read network to acquire information about the current network to manage
        let read_network = PolicyRule {
            api_groups: Some(vec![Network::group(&()).into()]),
            resources: Some(vec![Network::plural(&()).into()]),
            verbs: vec!["get".to_owned(), "watch".to_owned(), "list".to_owned()],
            ..Default::default()
        };

        // RATIONALE: manage tunnels to, well, manage tunnels (monitor state, delete when expired)
        let manage_tunnels = PolicyRule {
            api_groups: Some(vec![Tunnel::group(&()).into()]),
            resources: Some(vec![Tunnel::plural(&()).into()]),
            verbs: vec![
                "get".to_owned(),
                "watch".to_owned(),
                "list".to_owned(),
                "update".to_owned(),
                "patch".to_owned(),
                "delete".to_owned(),
            ],
            ..Default::default()
        };

        // RATIONALE: update tunnel statuses to fill relevant data for config generation for peers
        let update_tunnel_statuses = PolicyRule {
            api_groups: Some(vec![Tunnel::group(&()).into()]),
            resources: Some(vec![format!("{}/status", Tunnel::plural(&()))]),
            verbs: vec!["update".to_owned(), "patch".to_owned()],
            ..Default::default()
        };

        ClusterRole {
            metadata: self.generate_clusterwide_metadata(NETWORK_MANAGER_CLUSTERROLE_NAME),
            rules: Some(vec![read_network, manage_tunnels, update_tunnel_statuses]),
            ..Default::default()
        }
    }

    pub fn generate_router_clusterrole(&self) -> ClusterRole {
        // RATIONALE: get network to acquire information about the current network for this router
        let get_network = PolicyRule {
            api_groups: Some(vec![Network::group(&()).into()]),
            resources: Some(vec![Network::plural(&()).into()]),
            verbs: vec!["get".to_owned(), "watch".to_owned(), "list".to_owned()],
            ..Default::default()
        };

        // RATIONALE: read tunnels to generate peer configurations
        let read_tunnels = PolicyRule {
            api_groups: Some(vec![Tunnel::group(&()).into()]),
            resources: Some(vec![Tunnel::plural(&()).into()]),
            verbs: vec!["get".to_owned(), "watch".to_owned(), "list".to_owned()],
            ..Default::default()
        };

        ClusterRole {
            metadata: self.generate_clusterwide_metadata(ROUTER_CLUSTERROLE_NAME),
            rules: Some(vec![get_network, read_tunnels]),
            ..Default::default()
        }
    }
}
