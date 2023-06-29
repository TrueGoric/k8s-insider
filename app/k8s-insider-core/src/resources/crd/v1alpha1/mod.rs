use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    api::{DeleteParams, PatchParams},
    Client, CustomResourceExt,
};

use crate::{
    helpers::AndIf,
    kubernetes::operations::{apply_crd, try_remove_cluster_resource},
};

use self::{
    addressallocation::AddressAllocation, connection::Connection, network::Network, tunnel::Tunnel,
};

pub mod addressallocation;
pub mod connection;
pub mod network;
pub mod tunnel;

pub async fn create_v1alpha1_crds(
    client: &Client,
    apply_params: &PatchParams,
) -> anyhow::Result<()> {
    let network_spec = Network::crd();
    let addressallocation_spec = AddressAllocation::crd();
    let tunnel_spec = Tunnel::crd();
    let connection_spec = Connection::crd();

    apply_crd(client, &network_spec, apply_params).await?;
    apply_crd(client, &addressallocation_spec, apply_params).await?;
    apply_crd(client, &tunnel_spec, apply_params).await?;
    apply_crd(client, &connection_spec, apply_params).await?;

    Ok(())
}

pub async fn remove_v1alpha1_crds(client: &Client, dry_run: bool) -> anyhow::Result<()> {
    let delete_params = DeleteParams::foreground().and_if(dry_run, |p| p.dry_run());

    try_remove_cluster_resource::<CustomResourceDefinition>(
        client,
        Network::crd_name(),
        &delete_params,
    )
    .await?;
    try_remove_cluster_resource::<CustomResourceDefinition>(
        client,
        AddressAllocation::crd_name(),
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
