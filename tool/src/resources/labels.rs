use std::collections::BTreeMap;

use kube::api::ListParams;

pub fn get_common_labels() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("app.kubernetes.io/name".to_owned(), "k8s-insider".to_owned()),
        ("app.kubernetes.io/managed-by".to_owned(), "k8s-insider-cli".to_owned()),
    ])
}

pub fn get_joined_common_labels() -> String {
    format!("app.kubernetes.io/name=k8s-insider,\
            app.kubernetes.io/managed-by=k8s-insider-cli")
}

pub fn get_common_listparams() -> ListParams {
    ListParams::default().labels(&get_joined_common_labels())
}

pub fn get_release_labels(release_name: &str) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("k8s-insider/release-name".to_owned(), release_name.to_owned()),
        ("app.kubernetes.io/name".to_owned(), "k8s-insider".to_owned()),
        ("app.kubernetes.io/instance".to_owned(), format!("k8s-insider-{release_name}")),
        ("app.kubernetes.io/managed-by".to_owned(), "k8s-insider-cli".to_owned()),
    ])
}

pub fn get_joined_release_labels(release_name: &str) -> String {
    format!("k8s-insider/release-name={release_name},\
            app.kubernetes.io/name=k8s-insider,\
            app.kubernetes.io/instance=k8s-insider-{release_name},\
            app.kubernetes.io/managed-by=k8s-insider-cli")
}

pub fn get_release_listparams(release_name: &str) -> ListParams {
    ListParams::default().labels(&get_joined_release_labels(release_name))
}

pub fn get_release_selector_labels(release_name: &str) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("k8s-insider/release-name".to_owned(), release_name.to_owned()),
        ("app.kubernetes.io/name".to_owned(), "k8s-insider".to_owned()),
        ("app.kubernetes.io/instance".to_owned(), format!("k8s-insider-{release_name}")),
    ])
}

pub fn get_joined_release_selector_labels(release_name: &str) -> String {
    format!("k8s-insider/release-name={release_name},\
            app.kubernetes.io/name=k8s-insider,\
            app.kubernetes.io/instance=k8s-insider-{release_name}")
}

pub fn get_release_selector_listparams(release_name: &str) -> ListParams {
    ListParams::default().labels(&get_joined_release_selector_labels(release_name))
}