use k8s_openapi::api::core::v1::Namespace;
use kube::{api::{Patch, PatchParams}, core::ObjectMeta, Api, Client};

pub async fn create_namespace_if_not_exists(
    client: &Client,
    patch_params: &PatchParams,
    name: &str,
) -> anyhow::Result<()> {
    let namespace_api: Api<Namespace> = Api::all(client.clone());
    let namespace = Namespace {
        metadata: ObjectMeta {
            name: Some(name.to_owned()),
            ..Default::default()
        },
        ..Default::default()
    };

    namespace_api.patch(&name, patch_params, &Patch::Apply(namespace)).await?;

    Ok(())
}
