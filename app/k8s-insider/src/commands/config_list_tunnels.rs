use k8s_insider_macros::TableOutputRow;
use serde::Serialize;

use crate::{
    cli::ConfigListTunnelsArgs,
    config::{network::NetworkConfig, tunnel::TunnelConfig},
    context::ConfigContext,
    output::CliPrint,
};

pub fn config_list_tunnels(
    args: ConfigListTunnelsArgs,
    context: ConfigContext,
) -> anyhow::Result<()> {
    let config = context.insider_config();

    let tunnels = match args.network {
        Some(network) => config
            .try_get_network(&network)
            .iter()
            .flat_map(|n| {
                n.1.list_tunnels().map(|t| TunnelConfigViewSourceData {
                    local_network_name: n.0,
                    network: n.1,
                    local_tunnel_name: t.0,
                    tunnel: t.1,
                })
            })
            .collect::<Vec<_>>(),
        None => config
            .list_networks()
            .flat_map(|n| {
                n.1.list_tunnels().map(|t| TunnelConfigViewSourceData {
                    local_network_name: n.0,
                    network: n.1,
                    local_tunnel_name: t.0,
                    tunnel: t.1,
                })
            })
            .collect::<Vec<_>>(),
    };

    let tunnels = tunnels
        .into_iter()
        .map(std::convert::Into::<TunnelConfigView>::into)
        .collect::<Vec<_>>();

    tunnels.print(args.output)?;

    Ok(())
}

struct TunnelConfigViewSourceData<'a> {
    pub local_network_name: &'a String,
    pub local_tunnel_name: &'a String,
    pub network: &'a NetworkConfig,
    pub tunnel: &'a TunnelConfig,
}

#[derive(Serialize, TableOutputRow)]
struct TunnelConfigView<'a> {
    pub local_network_name: &'a str,
    #[name_column]
    pub local_name: &'a str,
    pub network: &'a str,
    pub context: &'a str,
    pub namespace: &'a str,
    pub crd_name: &'a str,
}

impl<'a> From<TunnelConfigViewSourceData<'a>> for TunnelConfigView<'a> {
    fn from(value: TunnelConfigViewSourceData<'a>) -> Self {
        TunnelConfigView {
            local_network_name: value.local_network_name,
            local_name: value.local_tunnel_name,
            network: &value.network.id.name,
            context: &value.network.id.context,
            namespace: &value.network.id.namespace,
            crd_name: &value.tunnel.name,
        }
    }
}
