use k8s_insider_core::{
    kubernetes::operations::list_resources, resources::crd::v1alpha1::network::Network,
};
use kube::{api::ListParams, Client};

use crate::cli::GlobalArgs;

pub async fn list_networks(global_args: GlobalArgs, client: Client) -> anyhow::Result<()> {
    let list_params = ListParams::default();
    let networks = list_resources::<Network>(&client, &global_args.namespace, &list_params).await?;

    for network in networks {
        println!("{}", network.metadata.name.unwrap());
    }

    Ok(())
}
