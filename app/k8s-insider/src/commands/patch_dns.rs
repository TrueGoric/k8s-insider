use anyhow::anyhow;
use k8s_insider_core::{
    kubernetes::operations::try_get_resource, resources::crd::v1alpha1::network::Network,
};
use log::info;

use crate::{cli::PatchDnsArgs, context::ConfigContext};

pub async fn patch_dns(args: PatchDnsArgs, context: ConfigContext) -> anyhow::Result<()> {
    let (_, network_config) = context
        .insider_config
        .get_network_or_default(args.network.as_deref())?;

    let client = context.create_client(&network_config.id.context).await?;
    let network = try_get_resource::<Network>(
        &client,
        &network_config.id.name,
        &network_config.id.namespace,
    )
    .await?
    .ok_or(anyhow!(
        "Couldn't find '{}' network on the cluster!",
        network_config.id.name
    ))?;
    let cluster_domain = network
        .status
        .ok_or(anyhow!(
            "Network '{}' is not in a ready state!",
            network_config.id.name
        ))?
        .service_domain
        .ok_or(anyhow!("Network's service domain is not defined!"))?;

    #[cfg(target_os = "linux")]
    {
        use crate::wireguard::operations::patch_dns_linux;

        let interface_name: &str = context
            .connections
            .get_wg_tunnel_ifname(&network_config.id)?;

        patch_dns_linux(interface_name, &cluster_domain)?;

        info!("Configured '{interface_name}' interface to handle DNS requests for '{cluster_domain}' domain with systemd-resolved!")
    }

    #[cfg(not(any(target_os = "linux")))]
    {
        return Err(anyhow!(
            "Can't patch DNS resolver! {} is an unsupported OS for this operation!",
            std::env::consts::OS
        ));
    }

    Ok(())
}
