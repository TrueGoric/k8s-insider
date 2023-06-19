use std::collections::BTreeMap;

use kube::api::ListParams;

pub fn get_agent_labels() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("app.kubernetes.io/name".to_owned(), "k8s-insider".to_owned()),
        ("app.kubernetes.io/component".to_owned(), "agent".to_owned()),
        ("app.kubernetes.io/managed-by".to_owned(), "k8s-insider-cli".to_owned()),
    ])
}

pub fn get_joined_agent_labels() -> String {
    format!("app.kubernetes.io/name=k8s-insider,\
            app.kubernetes.io/component=agent,\
            app.kubernetes.io/managed-by=k8s-insider-cli")
}

pub fn get_agent_listparams() -> ListParams {
    ListParams::default().labels(&get_joined_agent_labels())
}

pub fn get_tunnel_labels() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("app.kubernetes.io/name".to_owned(), "k8s-insider".to_owned()),
        ("app.kubernetes.io/component".to_owned(), "tunnel".to_owned()),
        ("app.kubernetes.io/managed-by".to_owned(), "k8s-insider-cli".to_owned()),
    ])
}

pub fn get_joined_tunnel_labels() -> String {
    format!("app.kubernetes.io/name=k8s-insider,\
            app.kubernetes.io/component=tunnel,\
            app.kubernetes.io/managed-by=k8s-insider-cli")
}


pub fn get_tunnel_listparams() -> ListParams {
    ListParams::default().labels(&get_joined_tunnel_labels())
}
