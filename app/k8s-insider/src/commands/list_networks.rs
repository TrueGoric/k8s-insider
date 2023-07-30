use std::net::SocketAddr;

use k8s_insider_core::{
    ip::{addrpair::IpAddrPair, netpair::IpNetPair, schema::IpNetFit},
    kubernetes::operations::list_resources,
    resources::crd::v1alpha1::network::{Network, NetworkService, NetworkState},
};
use k8s_insider_macros::TableOutputRow;
use kube::api::ListParams;
use serde::Serialize;

use crate::{
    cli::{GlobalArgs, ListNetworksArgs, OutputFormat},
    context::ConfigContext,
    output::{SerializableOutputDisplay, TableCellOption, TableCellSlice, TableOutputDisplay},
};

pub async fn list_networks(
    global_args: GlobalArgs,
    args: ListNetworksArgs,
    context: ConfigContext,
) -> anyhow::Result<()> {
    let client = context.create_client_with_default_context().await?;
    let list_params = ListParams::default();
    let networks = list_resources::<Network>(&client, &global_args.namespace, &list_params).await?;
    let network_views = networks
        .iter()
        .map(|n| n.into())
        .collect::<Vec<NetworkView>>();

    match args.output {
        OutputFormat::Names => network_views.print_names(),
        OutputFormat::Table => network_views.print_table(),
        OutputFormat::TableWithHeaders => network_views.print_table_with_headers(),
        OutputFormat::Json => network_views.print_json()?,
        OutputFormat::JsonPretty => network_views.print_json_pretty()?,
        OutputFormat::Yaml => network_views.print_yaml()?,
    };

    Ok(())
}

#[derive(Serialize, TableOutputRow)]
struct NetworkView<'a> {
    #[name_column]
    pub name: TableCellOption<&'a str>,
    pub peer_cidr: &'a IpNetPair,
    pub network_service_type: TableCellOption<&'a NetworkService>,
    pub server_public_key: TableCellOption<&'a str>,
    pub dns: TableCellOption<&'a IpAddrPair>,
    pub endpoints: TableCellOption<TableCellSlice<'a, SocketAddr>>,
    pub allowed_ips: TableCellOption<TableCellSlice<'a, IpNetFit>>,
    pub state: TableCellOption<&'a NetworkState>,
}

impl<'a> From<&'a Network> for NetworkView<'a> {
    fn from(value: &'a Network) -> Self {
        NetworkView {
            name: value.metadata.name.as_deref().into(),
            peer_cidr: &value.spec.peer_cidr,
            network_service_type: value.spec.network_service.as_ref().into(),
            server_public_key: value
                .status
                .as_ref()
                .and_then(|s| s.server_public_key.as_deref())
                .into(),
            dns: value.status.as_ref().and_then(|s| s.dns.as_ref()).into(),
            endpoints: value
                .status
                .as_ref()
                .and_then(|s| s.endpoints.as_ref())
                .map(|e| e.as_slice().into())
                .into(),
            allowed_ips: value
                .status
                .as_ref()
                .and_then(|s| s.allowed_ips.as_ref())
                .map(|e| e.as_slice().into())
                .into(),
            state: value.status.as_ref().map(|s| &s.state).into(),
        }
    }
}
