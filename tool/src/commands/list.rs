use anyhow::Context;
use k8s_openapi::api::apps::v1::Deployment;
use kube::{Api, Client};

use crate::{cli::GlobalArgs, resources::get_common_listparams};

pub async fn list(global_args: &GlobalArgs, client: &Client) -> anyhow::Result<()> {
    let releases_params = get_common_listparams();
    let deployment_api: Api<Deployment> = Api::namespaced(client.clone(), &global_args.namespace);
    let deployments = deployment_api
        .list_metadata(&releases_params)
        .await
        .context("Couldn't retrieve releases from the cluster!")?;

    for deployment in deployments {
        if let Some(name) = deployment.metadata.name {
            println!("{name}");
        }
    }

    Ok(())
}
