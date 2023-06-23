use std::collections::BTreeMap;

use kube::api::ListParams;

pub fn get_controller_labels() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "app.kubernetes.io/name".to_owned(),
            "k8s-insider".to_owned(),
        ),
        (
            "app.kubernetes.io/component".to_owned(),
            "controller".to_owned(),
        ),
        (
            "app.kubernetes.io/managed-by".to_owned(),
            "k8s-insider".to_owned(),
        ),
    ])
}

pub fn get_joined_controller_labels() -> String {
    "app.kubernetes.io/name=k8s-insider,\
            app.kubernetes.io/component=controller,\
            app.kubernetes.io/managed-by=k8s-insider"
        .to_string()
}

pub fn get_controller_listparams() -> ListParams {
    ListParams::default().labels(&get_joined_controller_labels())
}

pub fn get_router_labels() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "app.kubernetes.io/name".to_owned(),
            "k8s-insider".to_owned(),
        ),
        (
            "app.kubernetes.io/component".to_owned(),
            "router".to_owned(),
        ),
        (
            "app.kubernetes.io/managed-by".to_owned(),
            "k8s-insider".to_owned(),
        ),
    ])
}

pub fn get_joined_router_labels() -> String {
    "app.kubernetes.io/name=k8s-insider,\
            app.kubernetes.io/component=router,\
            app.kubernetes.io/managed-by=k8s-insider"
        .to_string()
}

pub fn get_router_listparams() -> ListParams {
    ListParams::default().labels(&get_joined_router_labels())
}
