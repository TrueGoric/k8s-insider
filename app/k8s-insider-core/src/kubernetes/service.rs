use std::net::{IpAddr, SocketAddr};

use itertools::chain;
use k8s_openapi::api::core::v1::{
    Node, NodeAddress, PortStatus, Service, ServiceSpec, ServiceStatus,
};

pub async fn get_service_accessible_addresses(
    service: Option<&Service>,
    nodes: &[&Node],
) -> Option<Vec<SocketAddr>> {
    let service = match service {
        Some(service) => service,
        None => return None,
    };
    let service_spec = match service.spec.as_ref() {
        Some(spec) => spec,
        None => return None,
    };
    let port = match get_first_port(service_spec) {
        Some(port) => port,
        None => return None,
    };

    // this is a simplified approach, a cartesian product may be more applicable
    let external_ips = service_spec
        .external_ips
        .as_deref()
        .unwrap_or_default()
        .iter()
        .filter_map(|raw| raw.parse::<IpAddr>().ok())
        .map(|addr| SocketAddr::new(addr, port));

    let service_kind = service_spec.type_.as_deref().unwrap_or("ClusterIP");

    match service_kind {
        "NodePort" => match get_nodeport_addresses(service_spec, nodes) {
            Some(additional_ips) => Some(chain![external_ips, additional_ips].collect()),
            None => Some(external_ips.collect()),
        },
        "LoadBalancer" => match get_loadbalancer_addresses(service_spec, service.status.as_ref()) {
            Some(additional_ips) => Some(chain![external_ips, additional_ips].collect()),
            None => Some(external_ips.collect()),
        },
        "ClusterIP" => match get_clusterip_addresses(service_spec) {
            Some(additional_ips) => Some(chain![external_ips, additional_ips].collect()),
            None => Some(external_ips.collect()),
        },
        _ => None,
    }
}

fn get_nodeport_addresses<'a>(
    service_spec: &'a ServiceSpec,
    nodes: &'a [&'a Node],
) -> Option<impl Iterator<Item = SocketAddr> + 'a> {
    let port = get_first_node_port(service_spec)?;

    // TODO: IPs from annotations

    let node_external_ips = get_node_address_iterator(nodes)
        .filter_map(|node_address| match node_address.type_.as_str() {
            "ExternalIP" => node_address.address.as_str().parse::<IpAddr>().ok(),
            _ => None,
        })
        .map(move |addr| SocketAddr::new(addr, port));
    let node_internal_ips = get_node_address_iterator(nodes)
        .filter_map(|node_address| match node_address.type_.as_str() {
            "InternalIP" => node_address.address.as_str().parse::<IpAddr>().ok(),
            _ => None,
        })
        .map(move |addr| SocketAddr::new(addr, port));

    Some(chain![node_external_ips, node_internal_ips])
}

fn get_loadbalancer_addresses<'a>(
    service_spec: &'a ServiceSpec,
    service_status: Option<&'a ServiceStatus>,
) -> Option<impl Iterator<Item = SocketAddr> + 'a> {
    service_status
        .and_then(|s| s.load_balancer.as_ref())
        .and_then(|l| l.ingress.as_ref())
        .map(|i| {
            i.iter().flat_map(|e| {
                let port = e
                    .ports
                    .as_ref()
                    .and_then(|p| get_first_loadbalancer_port(p))
                    .or_else(|| get_first_node_port(service_spec));

                e.ip.iter().filter_map(move |ip| {
                    let ip = ip.parse().ok()?;
                    let port = port?;
                    Some(SocketAddr::new(ip, port))
                })
            })
        })
}

fn get_clusterip_addresses(
    service_spec: &ServiceSpec,
) -> Option<impl Iterator<Item = SocketAddr> + '_> {
    let port = get_first_port(service_spec)?;

    service_spec.cluster_ips.as_ref().map(move |ips| {
        ips.iter().filter_map(move |ip| {
            let ip = ip.parse().ok()?;
            Some(SocketAddr::new(ip, port))
        })
    })
}

fn get_node_address_iterator<'a>(nodes: &'a [&'a Node]) -> impl Iterator<Item = &'a NodeAddress> {
    nodes
        .iter()
        .filter_map(|node| {
            node.status
                .as_ref()
                .and_then(|status| status.addresses.as_ref())
        })
        .flatten()
}

fn get_first_port(service_spec: &ServiceSpec) -> Option<u16> {
    service_spec
        .ports
        .as_ref()
        .and_then(|v| v.first())
        .map(|p| p.port as u16)
}

fn get_first_node_port(service_spec: &ServiceSpec) -> Option<u16> {
    service_spec
        .ports
        .as_ref()
        .and_then(|i| i.first())
        .and_then(|i| i.node_port)
        .map(|port| port as u16)
}

fn get_first_loadbalancer_port(ports: &[PortStatus]) -> Option<u16> {
    ports.iter().map(|p| p.port as u16).next()
}
