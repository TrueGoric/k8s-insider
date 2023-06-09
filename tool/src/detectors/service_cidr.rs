use std::time::{SystemTime, Duration};

use anyhow::anyhow;
use k8s_openapi::api::core::v1::{Service, ServiceSpec};
use kube::{Client, Api, api::PostParams, core::ObjectMeta};
use log::{info, debug};
use regex::Regex;

const DEFAULT_NAMESPACE: &str = "default";

pub async fn detect_service_cidr(client: &Client) -> anyhow::Result<String> {
    let services_api: Api<Service> = Api::namespaced(client.clone(), DEFAULT_NAMESPACE);
    let faux_service = get_faux_service();

    // why isn't there a dedicated API for that? ;_;
    let service_post_response = services_api
        .create(&PostParams::default(), &faux_service)
        .await;

    debug!("{service_post_response:?}");

    let service_cidr_regex: Regex =
        Regex::new("The range of valid IPs is (?<cidr>[0-9a-f./:]+)").unwrap();
    let service_cidr = match service_post_response {
        Ok(_) => panic!("Kubernetes accepted an invalid service definition - something is not right."),
        Err(error) => match error {
            kube::Error::Api(error) => Ok(service_cidr_regex
                .captures(&error.message)
                .ok_or(anyhow!(
                    "Couldn't retrieve valid service IPs from kubernetes API!"
                ))?
                .name("cidr")
                .ok_or(anyhow!(
                    "Couldn't retrieve valid service IPs from kubernetes API!"
                ))?
                .as_str()
                .to_owned()),
            _ => Err(error),
        },
    };

    let service_cidr = service_cidr?;

    info!("Detected service CIDR: {service_cidr}");

    Ok(service_cidr)
}

fn get_faux_service() -> Service {
    Service {
        metadata: ObjectMeta {
            name: Some(format!(
                "thisiswheredreamsgotodie{}",
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or(Duration::from_secs(2137420))
                    .as_secs()
            )),
            ..Default::default()
        },
        spec: Some(ServiceSpec {
            cluster_ip: Some("0.0.0.0".to_owned()),
            ..Default::default()
        }),
        status: None,
    }
}