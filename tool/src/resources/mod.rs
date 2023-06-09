use kube::api::ListParams;

pub mod release;

pub fn get_common_release_labels(release_name: &str) -> String {
    format!("k8s-insider/release-name={release_name},\
            app.kubernetes.io/name=k8s-insider,\
            app.kubernetes.io/instance=k8s-insider-{release_name},\
            app.kubernetes.io/managed-by=k8s-insider-cli")
}

pub fn get_common_release_listparams(release_name: &str) -> ListParams {
    ListParams::default().labels(&get_common_release_labels(release_name))
}