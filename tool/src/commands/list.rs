use anyhow::Context;
use k8s_openapi::api::apps::v1::Deployment;
use kube::Api;

use crate::{cli::GlobalArgs, kubernetes::create_client, resources::get_common_listparams};

pub async fn list(global_args: &GlobalArgs) -> anyhow::Result<()> {
    let client = create_client(&global_args.kube_config)
        .await
        .context("Couldn't initialize k8s API client!")?;

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
