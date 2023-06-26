use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    api::{DeleteParams, PatchParams},
    Client, CustomResourceExt,
};

use crate::{
    kubernetes::operations::{create_crd, try_remove_cluster_resource},
    resources::crd::v1alpha1::{connection::Connection, network::Network, tunnel::Tunnel},
    FIELD_MANAGER,
};

pub mod connection;
pub mod network;
pub mod tunnel;

pub async fn create_v1alpha1_crds(client: &Client, dry_run: bool) -> anyhow::Result<()> {
    let network_spec = Network::crd();
    let tunnel_spec = Tunnel::crd();
    let connection_spec = Connection::crd();

    let patch_params = PatchParams {
        field_manager: Some(FIELD_MANAGER.to_owned()),
        dry_run,
        ..Default::default()
    };

    create_crd(client, &network_spec, &patch_params).await?;
    create_crd(client, &tunnel_spec, &patch_params).await?;
    create_crd(client, &connection_spec, &patch_params).await?;

    Ok(())
}

pub async fn remove_v1alpha1_crds(client: &Client, dry_run: bool) -> anyhow::Result<()> {
    let mut delete_params = DeleteParams::foreground();

    if dry_run {
        delete_params = delete_params.dry_run();
    }

    try_remove_cluster_resource::<CustomResourceDefinition>(
        client,
        Network::crd_name(),
        &delete_params,
    )
    .await?;
    try_remove_cluster_resource::<CustomResourceDefinition>(
        client,
        Tunnel::crd_name(),
        &delete_params,
    )
    .await?;
    try_remove_cluster_resource::<CustomResourceDefinition>(
        client,
        Connection::crd_name(),
        &delete_params,
    )
    .await?;

    Ok(())
}
