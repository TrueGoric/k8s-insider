use insider_macros::TableOutputRow;
use serde::Serialize;

use crate::{
    cli::{ConfigListTunnelsArgs, OutputFormat},
    config::{network::NetworkConfig, tunnel::TunnelConfig, ConfigContext},
    output::{SerializableOutputDisplay, TableOutputDisplay},
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
                    network_name: n.0,
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
                    network_name: n.0,
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

    match args.output {
        OutputFormat::Names => tunnels.print_names(),
        OutputFormat::Table => tunnels.print_table(),
        OutputFormat::TableWithHeaders => tunnels.print_table_with_headers(),
        OutputFormat::Json => tunnels.print_json()?,
        OutputFormat::JsonPretty => tunnels.print_json_pretty()?,
        OutputFormat::Yaml => tunnels.print_yaml()?,
    }

    Ok(())
}

struct TunnelConfigViewSourceData<'a> {
    pub network_name: &'a String,
    pub local_tunnel_name: &'a String,
    pub network: &'a NetworkConfig,
    pub tunnel: &'a TunnelConfig,
}

#[derive(Serialize, TableOutputRow)]
struct TunnelConfigView<'a> {
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
            local_name: value.local_tunnel_name,
            network: value.network_name,
            context: &value.network.context,
            namespace: &value.network.namespace,
            crd_name: &value.tunnel.name,
        }
    }
}
