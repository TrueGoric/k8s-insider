use std::collections::HashMap;

use anyhow::anyhow;
use ipnet::Ipv4Net;
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{api::ListParams, Api, Client};
use log::{debug, info};

use crate::ip::netpair::IpNetPair;

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
    pub cidr: IpNetPair,
    pub cni: Cni,
}

pub async fn detect_pod_cidr(client: &Client) -> anyhow::Result<IpNetPair> {
    let configmaps = get_cni_configmaps(client).await?;

    if let Some(cni) = configmaps.first() {
        info!("Detected pod CIDR ({:?}): {}", cni.cni, cni.cidr);
        return Ok(cni.cidr);
    }

    Err(anyhow!(
        "Error detecting pod CIDR: unsupported CNI! Try passing the --pod-cidr parameter."
    ))
}

async fn get_cni_configmaps(client: &Client) -> anyhow::Result<Vec<CniCidr>> {
    let configmap_api: Api<ConfigMap> = Api::all(client.clone());

    // sorting through all configmaps in the cluster is not an ideal solution indeed
    // but CNI configs can be placed in a custom namespace and, by default,
    // may not be properly annotated
    let any_filter = ListParams::default();
    let configmaps = configmap_api
        .list(&any_filter)
        .await?
        .into_iter()
        .filter_map(|configmap| {
            configmap
                .metadata
                .name
                .as_ref()
                .and_then(|name| match name.as_str() {
                    CILIUM_CONFIGMAP_NAME => try_get_cilium_cidr(&configmap).map(|cidr| CniCidr {
                        cni: Cni::Cilium,
                        cidr: cidr.into(),
                    }),
                    FLANNEL_CONFIGMAP_NAME => {
                        try_get_flannel_cidr(&configmap).map(|cidr| CniCidr {
                            cni: Cni::Flannel,
                            cidr: cidr.into(),
                        })
                    }
                    _ => None,
                })
        })
        .collect();

    Ok(configmaps)
}

fn try_get_cilium_cidr(configmap: &ConfigMap) -> Option<Ipv4Net> {
    debug!("Found {CILIUM_CONFIGMAP_NAME} configmap!");

    configmap
        .data
        .as_ref()
        .or_else(|| {
            debug!("{CILIUM_CONFIGMAP_NAME} is missing the data section!");
            None
        })
        .and_then(|data| data.get(CILIUM_IPV4_CIDR_KEY))
        .or_else(|| {
            debug!("{CILIUM_CONFIGMAP_NAME} is missing the '{CILIUM_IPV4_CIDR_KEY}' key!");
            None
        })
        .and_then(|cidr| {
            cidr.parse::<Ipv4Net>()
                .map_err(|err| {
                    debug!("{CILIUM_IPV4_CIDR_KEY} is not a valid CIDR: {err}!");
                    err
                })
                .ok()
        })
}

fn try_get_flannel_cidr(configmap: &ConfigMap) -> Option<Ipv4Net> {
    debug!("Found {FLANNEL_CONFIGMAP_NAME} configmap!");

    configmap
        .data
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
        .and_then(|value| {
            serde_json::from_str::<HashMap<String, serde_json::Value>>(value).map_or_else(
                |err| {
                    debug!("{FLANNEL_NET_CONF_KEY} is not a valid JSON! {err:?}");
                    None
                },
                Some,
            )
        })
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
        .and_then(|cidr| {
            cidr.parse::<Ipv4Net>()
                .map_err(|err| {
                    debug!("{FLANNEL_NETWORK_CONF_PROPERTY} is not a valid CIDR: {err}");
                    err
                })
                .ok()
        })
}
