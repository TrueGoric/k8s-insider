use insider_macros::TableOutputRow;
use serde::Serialize;

use crate::{
    cli::ConfigListNetworksArgs,
    config::{network::NetworkConfig, ConfigContext},
    output::CliPrint,
};

pub fn config_list_networks(
    args: ConfigListNetworksArgs,
    context: ConfigContext,
) -> anyhow::Result<()> {
    let config = context.insider_config();

    let networks = config
        .list_networks()
        .map(|n| NetworkConfigViewSourceData {
            network_name: n.0,
            network: n.1,
        })
        .map(std::convert::Into::<NetworkConfigView>::into)
        .collect::<Vec<_>>();

    networks.print(args.output)?;

    Ok(())
}

struct NetworkConfigViewSourceData<'a> {
    pub network_name: &'a String,
    pub network: &'a NetworkConfig,
}

#[derive(Serialize, TableOutputRow)]
struct NetworkConfigView<'a> {
    #[name_column]
    pub name: &'a str,
    pub context: &'a str,
    pub namespace: &'a str,
}

impl<'a> From<NetworkConfigViewSourceData<'a>> for NetworkConfigView<'a> {
    fn from(value: NetworkConfigViewSourceData<'a>) -> Self {
        NetworkConfigView {
            name: value.network_name,
            context: &value.network.context,
            namespace: &value.network.namespace,
        }
    }
}
