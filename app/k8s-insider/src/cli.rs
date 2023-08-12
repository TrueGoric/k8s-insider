use std::net::Ipv4Addr;

use clap::{Args, Parser, Subcommand, ValueEnum};
use ipnet::Ipv4Net;

pub const DEFAULT_NAMESPACE: &str = "kube-insider";

pub const DEFAULT_PEER_CIDR: &str = "10.11.11.0/24";

pub const DEFAULT_CONTROLLER_IMAGE: &str = "ghcr.io/truegoric/k8s-insider-controller:latest";
pub const DEFAULT_NETWORK_MANAGER_IMAGE: &str =
    "ghcr.io/truegoric/k8s-insider-network-manager:latest";
pub const DEFAULT_ROUTER_IMAGE: &str = "ghcr.io/truegoric/k8s-insider-router:latest";

pub const DEFAULT_NETWORK_NAME: &str = "default";

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    #[command(flatten)]
    pub global_args: GlobalArgs,
}

#[derive(Debug, Args)]
pub struct GlobalArgs {
    /// kubernetes namespace to work with
    #[arg(short = 'n', long, global = true, default_value = DEFAULT_NAMESPACE)]
    pub namespace: String,
    /// path to the k8s-insider config file (defaults to 'insider-config' file in the user's kubeconfig directory)
    #[arg(long, global = true)]
    pub config: Option<String>,
    /// override default kubeconfig
    #[arg(long, global = true)]
    pub kube_config: Option<String>,
    /// override default kubeconfig context
    #[arg(long, global = true)]
    pub kube_context: Option<String>,
    /// enable verbose output
    #[arg(short = 'v', long = "verbose", global = true)]
    pub verbose_logging: bool,
    /// enable trace output (more detailed than verbose, overrides it if present)
    #[arg(long = "trace", global = true)]
    pub trace_logging: bool,
}

impl GlobalArgs {
    pub fn get_log_level(&self) -> LogLevel {
        if self.trace_logging {
            return LogLevel::Trace;
        }

        if self.verbose_logging {
            return LogLevel::Verbose;
        }

        LogLevel::Normal
    }
}

pub enum LogLevel {
    Normal,
    Verbose,
    Trace,
}

#[derive(Debug, Subcommand)]
#[command(arg_required_else_help = true)]
pub enum Commands {
    /// Install k8s-insider on the cluster
    #[command()]
    Install(InstallArgs),
    /// Uninstall k8s-insider from the cluster
    #[command()]
    Uninstall(UninstallArgs),
    /// Create a new network/tunnel on the cluster
    #[command()]
    Create(CreateCommand),
    /// Remove a network/tunnel from the cluster
    #[command(alias = "del")]
    Delete(DeleteCommand),
    /// List networks/tunnels
    #[command(alias = "ls")]
    List(ListCommand),
    /// Connect to a network
    #[command()]
    Connect(ConnectArgs),
    /// Disconnect from the network
    #[command()]
    Disconnect(DisconnectArgs),
    /// Get the WireGuard configuration file for a tunnel
    #[command()]
    GetConf(GetConfArgs),
    /// Patch the DNS resolver to avoid loops when deploying on the local machine
    #[command()]
    PatchDns(PatchDnsArgs),
    /// Modify k8s-insider configuration
    #[command(alias = "cfg", alias = "conf")]
    Config(ConfigCommand),
}

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// DNS service IP (autodetected if unset)
    #[arg(long)]
    pub kube_dns: Option<String>,
    /// Cluster service CIDR (autodetected if unset)
    #[arg(long)]
    pub service_cidr: Option<Ipv4Net>,
    /// Cluster domain name assigned to pods and services (autodetected if unset)
    #[arg(long)]
    pub service_domain: Option<String>,
    /// Cluster pod CIDR (autodetected if unset)
    #[arg(long)]
    pub pod_cidr: Option<Ipv4Net>,
    /// Don't install CRDs (should you choose not to install them here make sure beforehand they are available on the cluster)
    #[arg(long)]
    pub no_crds: bool,
    /// Substitutes the k8s-insider-controller container image if specified
    #[arg(long, default_value = DEFAULT_CONTROLLER_IMAGE)]
    pub controller_image: String,
    /// Substitutes the k8s-insider-controller container image if specified
    #[arg(long, default_value = DEFAULT_NETWORK_MANAGER_IMAGE)]
    pub network_manager_image: String,
    /// Substitutes the k8s-insider-router container image if specified
    #[arg(long, default_value = DEFAULT_ROUTER_IMAGE)]
    pub router_image: String,
    /// If set, no action will be taken on the cluster
    #[arg(long)]
    pub dry_run: bool,
    /// Force the installation
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct UninstallArgs {
    /// Try to remove the namespace afterwards
    #[arg(long)]
    pub delete_namespace: bool,
    /// don't remove CRDs
    #[arg(long)]
    pub leave_crds: bool,
    /// If set, no action will be taken on the cluster
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct CreateCommand {
    #[command(subcommand)]
    pub subcommand: CreateSubcommands,
}

#[derive(Debug, Subcommand)]
#[command(arg_required_else_help = true)]
pub enum CreateSubcommands {
    /// Create a new VPN network on the cluster
    #[command(alias = "n", alias = "net")]
    Network(CreateNetworkArgs),
    /// Create a new tunnel for a network (i.e. if you want to connect on another device)
    #[command(alias = "t", alias = "tun")]
    Tunnel(CreateTunnelArgs),
}

#[derive(Debug, Args)]
pub struct CreateNetworkArgs {
    /// Name of the network to create
    #[arg(default_value = DEFAULT_NETWORK_NAME)]
    pub name: String,
    /// Peer CIDR subnet (users will be assigned IPs from that range)
    #[arg(long, default_value = DEFAULT_PEER_CIDR)]
    pub peer_cidr: Ipv4Net,
    /// Type of the service that will be used to connect to the k8s-insider instance
    #[arg(long, value_enum, default_value_t = ServiceType::NodePort)]
    pub service_type: ServiceType,
    /// Manually sets the connection IPs (valid for NodePort and ExternalIp service types)
    ///
    /// Required for ExternalIp services,
    /// When defined with NodePort service type it skips the autodetection of node IPs and
    /// instructs clients to connect using the provided addresses.
    #[arg(long)]
    pub external_ip: Option<Vec<Ipv4Addr>>,
    /// Sets up a static cluster IP for the service
    #[arg(long)]
    pub cluster_ip: Option<Ipv4Addr>,
    /// If set, no action will be taken on the cluster
    #[arg(long)]
    pub dry_run: bool,
    /// Apply changes to the network if it already exists
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Clone, ValueEnum)]
#[value()]
pub enum ServiceType {
    #[value(name = "None")]
    None,
    #[value(name = "ClusterIp")]
    ClusterIp,
    #[value(name = "NodePort")]
    NodePort,
    #[value(name = "LoadBalancer")]
    LoadBalancer,
    #[value(name = "ExternalIp")]
    ExternalIp,
}

#[derive(Debug, Args)]
pub struct CreateTunnelArgs {
    /// Name of the network to join (can be omitted if there's only one network in the config)
    #[arg()]
    pub network: Option<String>,
    /// Locally scoped tunnel name to save in the configuration file (autogenerated by default)
    #[arg()]
    pub name: Option<String>,
    /// Defines a static IP for the tunnel, which is assigned dynamically otherwise
    #[arg(long)]
    pub static_ip: Option<Ipv4Addr>,
}

#[derive(Debug, Args)]
pub struct DeleteCommand {
    #[command(subcommand)]
    pub subcommand: DeleteSubcommands,
}

#[derive(Debug, Subcommand)]
#[command(arg_required_else_help = true)]
pub enum DeleteSubcommands {
    /// Remove a VPN network from the cluster
    #[command(alias = "n", alias = "net")]
    Network(DeleteNetworkArgs),
    /// Manually remove a tunnel
    #[command(alias = "t", alias = "tun")]
    Tunnel(DeleteTunnelArgs),
}

#[derive(Debug, Args)]
pub struct DeleteNetworkArgs {
    /// Name of the network to remove
    #[arg()]
    pub name: String,
    /// If set, no action will be taken on the cluster
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct DeleteTunnelArgs {
    /// Tunnel's parent network
    #[arg()]
    pub network: String,
    /// Name of the tunnel to remove
    #[arg()]
    pub tunnel: String,
    /// If set, no action will be taken on the cluster
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct ListCommand {
    #[command(subcommand)]
    pub subcommand: ListSubcommands,
}

#[derive(Debug, Subcommand)]
#[command(arg_required_else_help = true)]
pub enum ListSubcommands {
    /// List cluster VPN networks
    #[command(alias = "n", alias = "net", alias = "nets", alias = "networks")]
    Network(ListNetworksArgs),
    /// List network tunnels
    #[command(alias = "t", alias = "tun", alias = "tuns", alias = "tunnels")]
    Tunnel(ListTunnelsArgs),
}

#[derive(Debug, Args)]
pub struct ListNetworksArgs {
    /// Output format
    #[arg(short = 'o', long, value_enum, default_value_t = OutputFormat::Table)]
    pub output: OutputFormat,
}

#[derive(Debug, Args)]
pub struct ListTunnelsArgs {
    /// Limit the search to tunnels belonging to a particular network (optional)
    /// 
    /// The list is filtered locally due to limitations in k8s API.
    #[arg()]
    pub network: Option<String>,
    /// Output format
    #[arg(short = 'o', long, value_enum, default_value_t = OutputFormat::Table)]
    pub output: OutputFormat,
}

#[derive(Debug, Args)]
pub struct ConnectArgs {
    /// Parent network (can be omitted if there's only one network in the config)
    #[arg()]
    pub network: Option<String>,    
    /// Name of the tunnel to connect to (can be omitted if the parent network configuration contains only one tunnel)
    #[arg()]
    pub name: Option<String>,
}

#[derive(Debug, Args)]
pub struct DisconnectArgs {
    /// Network to disconnect from
    #[arg()]
    pub network: Option<String>,    
}

#[derive(Debug, Args)]
pub struct GetConfArgs {
    /// Parent network (can be omitted if there's only one network in the config)
    #[arg()]
    pub network: Option<String>,    
    /// Name of the tunnel to generate configuration for (can be omitted if the parent network configuration contains only one tunnel)
    #[arg()]
    pub tunnel: Option<String>,
    /// If set, the command will write the config to a file instead of stdout
    #[arg(short = 'o', long)]
    pub output: Option<String>,
}

#[derive(Debug, Args)]
pub struct PatchDnsArgs {
    /// Name of the connected network to patch (autodetected if there's only one connection)
    pub network: Option<String>,
}

#[derive(Debug, Args)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub subcommand: ConfigSubcommands,
}

#[derive(Debug, Subcommand)]
#[command(arg_required_else_help = true)]
pub enum ConfigSubcommands {
    /// Add existing networks/tunnels
    #[command(alias = "a")]
    Add(ConfigAddCommand),
    /// List networks or tunnels defined in the config
    #[command(alias = "l", alias = "ls")]
    List(ConfigListCommand),
    /// Remove a network or a tunnel from the configuration (but not from the cluster)
    #[command(alias = "r", alias = "rm")]
    Remove(ConfigRemoveCommand),
}

#[derive(Debug, Args)]
pub struct ConfigAddCommand {
    #[command(subcommand)]
    pub subcommand: ConfigAddSubcommands,
}

#[derive(Debug, Subcommand)]
#[command(arg_required_else_help = true)]
pub enum ConfigAddSubcommands {
    /// Add a cluster network reference
    #[command(alias = "n", alias = "net")]
    Network(ConfigAddNetworkArgs),
    /// Add an existing tunnel to the configuration (expects a base64-encoded peer private key to be supplied through stdin)
    #[command(alias = "t", alias = "tun")]
    Tunnel(ConfigAddTunnelArgs),
}

#[derive(Debug, Args)]
pub struct ConfigAddNetworkArgs {
    /// Name of network
    #[arg()]
    pub name: String,
    /// Locally scoped name to persist in the configuration file (autogenerated by default)
    #[arg()]
    pub local_name: Option<String>,
}

#[derive(Debug, Args)]
pub struct ConfigAddTunnelArgs {
    /// Name of the Tunnel CRD resource to generate configuration for
    #[arg()]
    pub name: String,
    /// Name of the network this tunnel belongs to
    #[arg()]
    pub network: String,
    /// Locally scoped name to persist in the configuration file (autogenerated by default)
    #[arg()]
    pub local_name: Option<String>,
}

#[derive(Debug, Args)]
pub struct ConfigListCommand {
    #[command(subcommand)]
    pub subcommand: ConfigListSubcommands,
}

#[derive(Debug, Subcommand)]
#[command(arg_required_else_help = true)]
pub enum ConfigListSubcommands {
    /// List config networks
    #[command(alias = "n", alias = "net", alias = "nets", alias = "networks")]
    Network(ConfigListNetworksArgs),
    /// List an existing tunnel to the configuration (expects a base64-encoded peer private key to be supplied through stdin)
    #[command(alias = "t", alias = "tun", alias = "tuns", alias = "tunnels")]
    Tunnel(ConfigListTunnelsArgs),
}

#[derive(Debug, Args)]
pub struct ConfigListNetworksArgs {
    /// Output format
    #[arg(short = 'o', long, value_enum, default_value_t = OutputFormat::Table)]
    pub output: OutputFormat,
}

#[derive(Debug, Args)]
pub struct ConfigListTunnelsArgs {
    /// Limit the search to tunnels belonging to a particular network (optional)
    #[arg()]
    pub network: Option<String>,
    /// Output format
    #[arg(short = 'o', long, value_enum, default_value_t = OutputFormat::Table)]
    pub output: OutputFormat,
}

#[derive(Debug, Args)]
pub struct ConfigRemoveCommand {
    #[command(subcommand)]
    pub subcommand: ConfigRemoveSubcommands,
}

#[derive(Debug, Subcommand)]
#[command(arg_required_else_help = true)]
pub enum ConfigRemoveSubcommands {
    /// Remove a network and all associated tunnels from config
    #[command(alias = "n", alias = "net")]
    Network(ConfigRemoveNetworkArgs),
    /// Remove a tunnel from configuration
    #[command(alias = "t", alias = "tun")]
    Tunnel(ConfigRemoveTunnelArgs),
}

#[derive(Debug, Args)]
pub struct ConfigRemoveNetworkArgs {
    /// Name of the network to remove
    #[arg()]
    pub name: String,
}

#[derive(Debug, Args)]
pub struct ConfigRemoveTunnelArgs {
    /// Locally scoped name of the tunnel to remove
    #[arg()]
    pub name: String,
    /// Parent network of the tunnel
    #[arg()]
    pub network: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Names,
    Table,
    TableWithHeaders,
    Json,
    JsonPretty,
    Yaml,
}
