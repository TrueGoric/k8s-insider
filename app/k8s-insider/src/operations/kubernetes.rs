use anyhow::{anyhow, Context};
use k8s_openapi::api::core::v1::{Pod, Service, ServiceSpec};
use kube::{
    api::{AttachParams, ListParams},
    config::{KubeConfigOptions, Kubeconfig},
    Api, Client, Config,
};
use log::warn;
use serde::de::DeserializeOwned;
use tokio::io::AsyncReadExt;

pub async fn create_client(
    config_path: &Option<String>,
    context_name: &Option<String>,
) -> anyhow::Result<Client> {
    let config_options = KubeConfigOptions {
        context: context_name.to_owned(),
        ..Default::default()
    };

    let config = match config_path {
        Some(path) => {
            let kubeconfig = Kubeconfig::read_from(path)?;
            Config::from_custom_kubeconfig(kubeconfig, &config_options).await?
        }
        None => Config::from_kubeconfig(&config_options).await?,
    };

    let client = Client::try_from(config)?;

    Ok(client)
}

pub async fn get_text_file(
    client: &Client,
    namespace: &str,
    pod: &str,
    file_path: &str,
) -> anyhow::Result<String> {
    let pod_api: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let attach_params = AttachParams {
        stdout: true,
        stderr: true,
        ..Default::default()
    };

    let command = vec!["cat", &file_path];
    let mut exec = pod_api.exec(pod, command, &attach_params).await?;
    let mut file = String::new();

    let read_bytes = exec
        .stdout()
        .ok_or_else(|| anyhow!("Couldn't retrieve the remote process standard output!"))?
        .read_to_string(&mut file)
        .await?;

    if read_bytes == 0 {
        let mut error = String::new();

        exec.stderr()
            .ok_or_else(|| anyhow!("Couldn't retrieve the remote process standard error!"))?
            .read_to_string(&mut error)
            .await?;

        return Err(anyhow!("Couldn't read the file! Error: {error}"));
    }

    Ok(file)
}

pub async fn check_if_resource_exists<T: Clone + DeserializeOwned + core::fmt::Debug>(
    release_params: &ListParams,
    api: &Api<T>,
) -> anyhow::Result<bool> {
    let matching_deployments = api
        .list_metadata(&release_params)
        .await
        .context("Couldn't retrieve resources from the cluster!")?;

    match matching_deployments.items.len() {
        0 => Ok(false),
        1 => Ok(true),
        _ => {
            warn!("There are multiple resources matching the release! This could cause unintented behavior!");
            Ok(true)
        }
    }
}

pub async fn get_service_accessible_address(client: &Client, service: &Service) -> anyhow::Result<String> {
    let service_spec = service
        .spec
        .as_ref()
        .ok_or_else(|| anyhow!("Service is missing a specification!"))?;

    let service_external_ip = service_spec
        .external_ips
        .as_ref()
        .and_then(|vec| vec.first());

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
    Ok(service_spec
        .ports
        .as_ref()
        .ok_or_else(|| anyhow!("Service is missing ports!"))?
        .first()
        .ok_or_else(|| anyhow!("Service is missing ports!"))?
        .node_port
        .ok_or_else(|| anyhow!("Service is missing a node port!"))?)
}
