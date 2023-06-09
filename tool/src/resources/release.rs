use derive_builder::Builder;

#[derive(Debug, Builder)]
pub struct ReleaseInfo {
    pub release_name: String,
    pub kube_dns: Option<String>,
    pub service_cidr: String,
    pub service_domain: Option<String>,
    pub pod_cidr: String,
}