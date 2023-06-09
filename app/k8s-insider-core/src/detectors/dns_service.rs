use std::net::Ipv4Addr;

use anyhow::{anyhow, Context};
use k8s_openapi::api::core::v1::Service;
use kube::{Api, Client};
use log::{info, warn};

use crate::ip::addrpair::IpAddrPair;

const KUBE_DNS_SERVICE_NAME: &str = "kube-dns";
const KUBE_DNS_SERVICE_NAMESPACE: &str = "kube-system";

pub async fn detect_dns_service(client: &Client) -> anyhow::Result<Option<IpAddrPair>> {
    let services_api: Api<Service> = Api::namespaced(client.clone(), KUBE_DNS_SERVICE_NAMESPACE);
    let dns_service = services_api
        .get_opt(KUBE_DNS_SERVICE_NAME)
        .await
        .context("DNS service IP autodetection failed!")?;

    if dns_service.is_none() {
        warn!("Couldn't autodetect DNS service! DNS resolution will be unavailable.");

        return Ok(None);
    }

    let dns_service = dns_service
        .unwrap()
        .spec
        .ok_or(anyhow!("Missing spec for {KUBE_DNS_SERVICE_NAME} service!"))?
        .cluster_ip
        .ok_or(anyhow!(
            "Missing clusterIP for {KUBE_DNS_SERVICE_NAME} service!"
        ))?
        .parse::<Ipv4Addr>()?
        .into();

    info!("Detected DNS service IP: {dns_service}");

    Ok(Some(dns_service))
}
