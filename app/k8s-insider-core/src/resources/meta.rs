use super::{crd::v1alpha1::network::Network, router::RouterRelease};

pub trait NetworkMeta {
    fn get_router_name(&self) -> String;
    fn get_network_manager_name(&self) -> String;
    fn get_router_namespace(&self) -> String;
}

pub trait TryNetworkMeta {
    fn try_get_router_name(&self) -> Option<String>;
    fn try_get_network_manager_name(&self) -> Option<String>;
    fn try_get_router_namespace(&self) -> Option<String>;
}

impl NetworkMeta for RouterRelease {
    fn get_router_name(&self) -> String {
        format!("k8s-insider-router-{}", self.name)
    }

    fn get_network_manager_name(&self) -> String {
        format!("k8s-insider-network-manager-{}", self.name)
    }

    fn get_router_namespace(&self) -> String {
        self.namespace.to_owned()
    }
}

impl TryNetworkMeta for Network {
    fn try_get_router_name(&self) -> Option<String> {
        self.metadata
            .name
            .as_ref()
            .map(|name| format!("k8s-insider-router-{}", name))
    }

    fn try_get_network_manager_name(&self) -> Option<String> {
        self.metadata
            .name
            .as_ref()
            .map(|name| format!("k8s-insider-network-manager-{}", name))
    }

    fn try_get_router_namespace(&self) -> Option<String> {
        self.metadata.namespace.to_owned()
    }
}
