use std::collections::HashMap;

use anyhow::anyhow;
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{api::ListParams, Api, Client};
use log::{debug, info};

const CILIUM_CONFIGMAP_NAME: &str = "cilium-config";
const CILIUM_IPV4_CIDR_KEY: &str = "cluster-pool-ipv4-cidr";

const FLANNEL_CONFIGMAP_NAME: &str = "kube-flannel-cfg";
const FLANNEL_NET_CONF_KEY: &str = "net-conf.json";
const FLANNEL_NETWORK_CONF_PROPERTY: &str = "Network";

#[derive(Debug)]
enum Cni {
    Cilium,
    Flannel,
}

#[derive(Debug)]
struct CniCidr {
    pub cidr: String,
    pub cni: Cni,
}

pub async fn detect_pod_cidr(client: &Client) -> anyhow::Result<String> {
    let configmaps = get_cluster_configmaps(client).await?;

    for cni in configmaps {
        info!("Detected pod CIDR ({:?}): {}", cni.cni, cni.cidr);
        return Ok(cni.cidr);
    }

    Err(anyhow!(
        "Error detecting pod CIDR: unsupported CNI! Try passing the --pod-cidr parameter."
    ))
}

async fn get_cluster_configmaps(client: &Client) -> anyhow::Result<Vec<CniCidr>> {
    let configmap_api: Api<ConfigMap> = Api::all(client.clone());

    // sorting through all configmaps in the cluster is not an ideal solution indeed
    // but CNI configs can be placed in a custom namespace and, by default,
    // may not be properly annotated
    let any_filter = ListParams::default();
    let configmaps = configmap_api
        .list(&any_filter)
        .await?
        .into_iter()
        .filter_map(|configmap| match &configmap.metadata.name {
            Some(name) => match name.as_str() {
                CILIUM_CONFIGMAP_NAME => match try_get_cilium_cidr(&configmap) {
                    Some(cidr) => Some(CniCidr {
                        cni: Cni::Cilium,
                        cidr,
                    }),
                    None => None,
                },
                FLANNEL_CONFIGMAP_NAME => match try_get_flannel_cidr(&configmap) {
                    Some(cidr) => Some(CniCidr {
                        cni: Cni::Flannel,
                        cidr,
                    }),
                    None => None,
                },
                _ => None,
            },
            None => None,
        })
        .collect();

    Ok(configmaps)
}

fn try_get_cilium_cidr(configmap: &ConfigMap) -> Option<String> {
    debug!("Found {CILIUM_CONFIGMAP_NAME} configmap!");

    match &configmap.data {
        Some(data) => match &data.get(CILIUM_IPV4_CIDR_KEY) {
            Some(value) => Some((*value).to_owned()),
            None => {
                debug!("{CILIUM_CONFIGMAP_NAME} is missing the '{CILIUM_IPV4_CIDR_KEY}' key!");
                None
            }
        },
        None => {
            debug!("{CILIUM_CONFIGMAP_NAME} is missing the data section!");
            None
        }
    }
}

fn try_get_flannel_cidr(configmap: &ConfigMap) -> Option<String> {
    debug!("Found {FLANNEL_CONFIGMAP_NAME} configmap!");

    configmap.data
        .as_ref()
        .or_else(|| {
            debug!("{FLANNEL_CONFIGMAP_NAME} is missing the data section!");
            None
        })
        .and_then(|data| data.get(FLANNEL_NET_CONF_KEY))
        .or_else(|| {
            debug!("{FLANNEL_CONFIGMAP_NAME} is missing the '{FLANNEL_NET_CONF_KEY}' key!");
            None
        })
        .and_then(|value| serde_json::from_str::<HashMap<String, serde_json::Value>>(value).map_or_else(|err|{
            debug!("{FLANNEL_NET_CONF_KEY} is not a valid JSON! {err:?}");
            None    
        }, |val| Some(val)))
        .as_ref()
        .and_then(|conf| conf.get(FLANNEL_NETWORK_CONF_PROPERTY))
        .or_else(|| {
            debug!("{FLANNEL_NETWORK_CONF_PROPERTY} is missing from {FLANNEL_NET_CONF_KEY}!");
            None    
        })
        .and_then(|value| value.as_str())
        .or_else(|| {
            debug!("{FLANNEL_NETWORK_CONF_PROPERTY} is not a valid CIDR!");
            None    
        })
        .and_then(|cidr| Some(cidr.to_owned()))
}
