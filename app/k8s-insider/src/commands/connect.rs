use anyhow::{anyhow, Context};
use k8s_openapi::api::apps::v1::Deployment;
use kube::{Api, Client};

use crate::{
    cli::{ConnectArgs, GlobalArgs},
    operations::wg_config::get_peer_config,
    resources::labels::{get_common_listparams, get_release_listparams},
};

pub async fn connect(
    global_args: &GlobalArgs,
    args: &ConnectArgs,
    client: &Client,
) -> anyhow::Result<()> {
    let releases_params = match &args.release_name {
        Some(name) => get_release_listparams(name),
        None => get_common_listparams(),
    };

    let wg_config = get_peer_config(
        client,
        &args.release_name,
        &releases_params,
        &global_args.namespace,
    )
    .await
    .context("Couldn't retrieve WireGuard configuration!")?;

    Ok(())
}
