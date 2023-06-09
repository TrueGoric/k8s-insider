use anyhow::anyhow;
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{api::ListParams, Api, Client};
use log::info;

const CILIUM_CONFIGMAP_NAME: &str = "cilium-config";
const CILIUM_IPV4_CIDR_KEY: &str = "cluster-pool-ipv4-cidr";

pub async fn detect_pod_cidr(client: &Client) -> anyhow::Result<String> {
    if let Some(cidr) = try_get_cilium_cidr(client).await? {
        info!("Detected pod CIDR (Cilium): {cidr}");
        return Ok(cidr);
    }

    Err(anyhow!(
        "Error detecting pod CIDR: unsupported CNI! Try passing the --pod-cidr parameter."
    ))
}

async fn try_get_cilium_cidr(client: &Client) -> anyhow::Result<Option<String>> {
    let configmap_api: Api<ConfigMap> = Api::all(client.clone());

    // sorting through all configmaps in the cluster is not an ideal solution indeed
    // but cilium-config can be placed in a custom namespace and, by default,
    // isn't properly annotated
    let filter = ListParams::default();
    let cilium_configmap = configmap_api.list(&filter).await?;
    let cilium_cidr = cilium_configmap
        .iter()
        .filter(|configmap| match &configmap.metadata.name {
            Some(value) => value == CILIUM_CONFIGMAP_NAME,
            None => false,
        })
        .find_map(|configmap| match &configmap.data {
            Some(data) => match &data.get(CILIUM_IPV4_CIDR_KEY) {
                Some(value) => Some((*value).to_owned()),
                None => None,
            },
            None => None,
        });

    Ok(cilium_cidr)
}
