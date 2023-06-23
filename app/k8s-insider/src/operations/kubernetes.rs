use anyhow::anyhow;
use k8s_openapi::api::core::v1::{Service, ServiceSpec};
use kube::Client;

pub async fn get_service_accessible_address(
    client: &Client,
    service: &Service,
) -> anyhow::Result<String> {
    let service_spec = service
        .spec
        .as_ref()
        .ok_or_else(|| anyhow!("Service is missing a specification!"))?;

    let service_external_ip = service_spec
        .external_ips
        .as_ref()
        .and_then(|vec: &Vec<String>| vec.first());

    if let Some(ip) = service_external_ip {
        let port = get_first_port(service_spec)?;

        return Ok(format!("{ip}:{port}"));
    }

    let service_kind = service_spec
        .type_
        .as_ref()
        .ok_or_else(|| anyhow!("Service is missing the type property!"))?
        .as_str();

    match service_kind {
        "NodePort" => todo!(),
        "LoadBalancer" => todo!(),
        _ => Err(anyhow!("Unsupported service type ({service_kind})!")),
    }
}

fn get_first_port(service_spec: &ServiceSpec) -> anyhow::Result<i32> {
    Ok(service_spec
        .ports
        .as_ref()
        .ok_or_else(|| anyhow!("Service is missing ports!"))?
        .first()
        .ok_or_else(|| anyhow!("Service is missing ports!"))?
        .port)
}

fn get_first_node_port(service_spec: &ServiceSpec) -> anyhow::Result<i32> {
    service_spec
        .ports
        .as_ref()
        .ok_or_else(|| anyhow!("Service is missing ports!"))?
        .first()
        .ok_or_else(|| anyhow!("Service is missing ports!"))?
        .node_port
        .ok_or_else(|| anyhow!("Service is missing a node port!"))
}
