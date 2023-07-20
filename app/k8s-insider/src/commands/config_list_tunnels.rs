use insider_macros::TableOutputRow;
use serde::Serialize;

use crate::{
    cli::{ConfigListTunnelsArgs, OutputFormat},
    config::{tunnel::TunnelConfig, ConfigContext},
    output::{SerializableOutputDisplay, TableOutputDisplay},
};

pub fn config_list_tunnels(
    args: ConfigListTunnelsArgs,
    context: ConfigContext,
) -> anyhow::Result<()> {
    let tunnels = context
        .insider_config()
        .tunnels
        .iter()
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

#[derive(Serialize, TableOutputRow)]
struct TunnelConfigView<'a> {
    #[name_column]
    pub local_name: &'a str,
    pub context: &'a str,
    pub namespace: &'a str,
    pub crd_name: &'a str,
}

impl<'a> From<(&'a String, &'a TunnelConfig)> for TunnelConfigView<'a> {
    fn from(value: (&'a String, &'a TunnelConfig)) -> Self {
        TunnelConfigView {
            local_name: value.0,
            context: &value.1.context,
            namespace: &value.1.namespace,
            crd_name: &value.1.name,
        }
    }
}
