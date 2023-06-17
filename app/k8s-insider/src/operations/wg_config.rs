use anyhow::{anyhow, Context};
use k8s_insider_core::helpers::get_secs_since_unix_epoch;
use k8s_openapi::api::core::v1::Pod;
use kube::{api::ListParams, Api, Client};

use super::kubernetes::get_text_file;

const WG_PEER_CONFIG_PATH: &str = "/config/peer.conf";

pub async fn get_peer_config(
    client: &Client,
    release_name: &Option<String>,
    releases_params: &ListParams,
    namespace: &str,
) -> anyhow::Result<String> {
    let pod_api: Api<Pod> = Api::namespaced(client.clone(), namespace);

    let pods: kube::core::ObjectList<kube::core::PartialObjectMeta<Pod>> = pod_api
        .list_metadata(&releases_params)
        .await
        .context("Couldn't retrieve release pods from the cluster!")?;

    if release_name.is_none() && pods.items.len() > 1 {
        return Err(anyhow!("Multiple releases detected on the cluster! Please specify the release you want to connect to."));
    }

    let random_pod_number = get_secs_since_unix_epoch() as usize % pods.items.len(); // good enough
    let pod = pods.items[random_pod_number]
        .metadata
        .name
        .as_ref()
        .ok_or_else(|| anyhow!("Cluster returned a pod without a name!"))?;

    Ok(get_text_file(client, &namespace, pod, WG_PEER_CONFIG_PATH).await?)
}
