use std::collections::HashMap;

use anyhow::{anyhow, Context};
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{Api, Client};
use log::info;
use serde_yaml::Value;

const KUBELET_CONFIGMAP_NAME: &str = "kubelet-config";
const KUBELET_CONFIGMAP_NAMESPACE: &str = "kube-system";

const KUBELET_KUBELET_CONFIG_ENTRY: &str = "kubelet";
const KUBELET_CLUSTER_DOMAIN_CONFIG_ENTRY: &str = "clusterDomain";

pub async fn detect_cluster_domain(client: &Client) -> anyhow::Result<Option<String>> {
    let configmap_api: Api<ConfigMap> =
        Api::namespaced(client.clone(), KUBELET_CONFIGMAP_NAMESPACE);
    let kubelet_configmap = configmap_api
        .get_opt(KUBELET_CONFIGMAP_NAME)
        .await
        .context("Cluster domain autodetection failed!")?;

    if kubelet_configmap.is_none() {
        return Ok(None);
    }

    let kubelet_configmap = kubelet_configmap.unwrap().data.ok_or(anyhow!(
        "Missing data for {KUBELET_CONFIGMAP_NAME} configmap!"
    ))?;

    let kubelet_configmap = kubelet_configmap.get(KUBELET_KUBELET_CONFIG_ENTRY).ok_or(anyhow!(
        "Missing '{KUBELET_KUBELET_CONFIG_ENTRY}' data entry for {KUBELET_CONFIGMAP_NAME} configmap!"
    ))?;

    let kubelet_configmap = serde_yaml::from_str::<HashMap<String, Value>>(kubelet_configmap)?;
    let cluster_domain = kubelet_configmap
        .get(KUBELET_CLUSTER_DOMAIN_CONFIG_ENTRY)
        .ok_or(anyhow!(
            "Missing '{KUBELET_CLUSTER_DOMAIN_CONFIG_ENTRY}' config entry in kubelet configuration!"
        ))?
        .as_str()
        .ok_or(anyhow!(
            "Invalid {KUBELET_CLUSTER_DOMAIN_CONFIG_ENTRY} value!"
        ))?
        .to_owned();

    info!("Detected cluster domain: {cluster_domain}");

    Ok(Some(cluster_domain))
}
