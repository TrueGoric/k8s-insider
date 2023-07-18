use std::collections::BTreeMap;

use crate::{
    cli::{ConfigListTunnelsArgs, OutputFormat},
    config::{tunnel::TunnelConfig, ConfigContext},
};

pub fn config_list_tunnels(
    args: ConfigListTunnelsArgs,
    context: ConfigContext,
) -> anyhow::Result<()> {
    let tunnels = &context.insider_config().tunnels;
    match args.output {
        OutputFormat::Names => names_output(tunnels),
        OutputFormat::Table => table_output(tunnels),
        OutputFormat::TableWithHeaders => table_with_headers_output(tunnels),
        OutputFormat::Json => json_output(tunnels)?,
        OutputFormat::JsonPretty => json_pretty_output(tunnels)?,
        OutputFormat::Yaml => yaml_output(tunnels)?,
    }

    Ok(())
}

fn names_output(tunnels: &BTreeMap<String, TunnelConfig>) {
    for tunnel in tunnels {
        println!("{}", tunnel.0);
    }
}

fn table_output(tunnels: &BTreeMap<String, TunnelConfig>) {
    for tunnel in tunnels {
        println!(
            "{}\t{}\t{}\t{}",
            tunnel.0, tunnel.1.context, tunnel.1.namespace, tunnel.1.name
        );
    }
}

fn table_with_headers_output(tunnels: &BTreeMap<String, TunnelConfig>) {
    println!("LOCAL_NAME\tKUBE_CONTEXT\tNAMESPACE\tCRD_NAME");
    table_output(tunnels);
}

fn json_output(tunnels: &BTreeMap<String, TunnelConfig>) -> anyhow::Result<()> {
    let output = serde_json::to_string(tunnels)?;
    print!("{output}");

    Ok(())
}

fn json_pretty_output(tunnels: &BTreeMap<String, TunnelConfig>) -> anyhow::Result<()> {
    let output = serde_json::to_string_pretty(tunnels)?;
    print!("{output}");

    Ok(())
}

fn yaml_output(tunnels: &BTreeMap<String, TunnelConfig>) -> anyhow::Result<()> {
    let output = serde_yaml::to_string(tunnels)?;
    print!("{output}");

    Ok(())
}
