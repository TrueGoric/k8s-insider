use std::{collections::BTreeMap, net::IpAddr};

pub fn get_service_annotations(ips: &[IpAddr]) -> BTreeMap<String, String> {
    BTreeMap::from([(
        "k8s-insider/external-ip".to_owned(),
        ips.iter()
            .map(|ip| ip.to_string())
            .collect::<Vec<String>>()
            .join(","),
    )])
}
