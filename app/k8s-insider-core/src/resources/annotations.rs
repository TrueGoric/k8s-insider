use std::collections::BTreeMap;

pub fn get_service_annotations(ips: &str) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("k8s-insider/external-ip".to_owned(), ips.to_owned()),
    ])
}