use k8s_insider_core::{
    ip::addrpair::IpAddrPair,
    kubernetes::operations::list_resources,
    resources::crd::v1alpha1::tunnel::{Tunnel, TunnelState},
};
use k8s_insider_macros::TableOutputRow;
use kube::api::ListParams;
use serde::Serialize;

use crate::{
    cli::{GlobalArgs, ListTunnelsArgs, OutputFormat},
    context::ConfigContext,
    output::{SerializableOutputDisplay, TableCellOption, TableOutputDisplay},
};

pub async fn list_tunnels(
    global_args: GlobalArgs,
    args: ListTunnelsArgs,
    context: ConfigContext,
) -> anyhow::Result<()> {
    let client = context.create_client_with_default_context().await?;
    let list_params = ListParams::default();
    let tunnels = list_resources::<Tunnel>(&client, &global_args.namespace, &list_params).await?;
    let tunnel_views = tunnels.iter();
    let tunnel_views = match args.network {
        Some(ref network) => tunnel_views
            .filter(|t| &t.spec.network == network)
            .map(|t| t.into())
            .collect::<Vec<TunnelView>>(),
        None => tunnel_views.map(|t| t.into()).collect::<Vec<TunnelView>>(),
    };

    match args.output {
        OutputFormat::Names => tunnel_views.print_names(),
        OutputFormat::Table => tunnel_views.print_table(),
        OutputFormat::TableWithHeaders => tunnel_views.print_table_with_headers(),
        OutputFormat::Json => tunnel_views.print_json()?,
        OutputFormat::JsonPretty => tunnel_views.print_json_pretty()?,
        OutputFormat::Yaml => tunnel_views.print_yaml()?,
    };

    Ok(())
}

#[derive(Serialize, TableOutputRow)]
struct TunnelView<'a> {
    pub network: &'a str,
    #[name_column]
    pub name: TableCellOption<&'a str>,
    pub peer_public_key: &'a str,
    pub preshared_key: &'a str,
    pub requested_static_ip: TableCellOption<&'a IpAddrPair>,
    pub current_address: TableCellOption<&'a IpAddrPair>,
    pub state: TableCellOption<&'a TunnelState>,
}

impl<'a> From<&'a Tunnel> for TunnelView<'a> {
    fn from(value: &'a Tunnel) -> Self {
        TunnelView {
            network: &value.spec.network,
            name: value.metadata.name.as_deref().into(),
            peer_public_key: &value.spec.peer_public_key,
            preshared_key: &value.spec.preshared_key,
            requested_static_ip: value.spec.static_ip.as_ref().into(),
            current_address: value
                .status
                .as_ref()
                .and_then(|s| s.address.as_ref())
                .into(),
            state: value.status.as_ref().map(|s| &s.state).into(),
        }
    }
}
