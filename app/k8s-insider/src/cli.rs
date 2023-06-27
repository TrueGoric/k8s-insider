use std::net::Ipv4Addr;

use clap::{Args, Parser, Subcommand, ValueEnum};
use ipnet::Ipv4Net;

pub const DEFAULT_NAMESPACE: &str = "kube-insider";

pub const DEFAULT_PEER_CIDR: &str = "10.11.11.0/24";

pub const DEFAULT_ROUTER_IMAGE: &str = "ghcr.io/truegoric/k8s-insider-router:latest";
pub const DEFAULT_CONTROLLER_IMAGE: &str = "ghcr.io/truegoric/k8s-insider-controller:latest";

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
    #[command(alias = "i")]
    Install(InstallArgs),
    /// Uninstall k8s-insider from the cluster
    #[command(alias = "u")]
    Uninstall(UninstallArgs),
    /// Create a new VPN network on the cluster
    #[command(alias = "n", alias = "create")]
    CreateNetwork(CreateNetworkArgs),
    /// Remove a VPN network from the cluster
    #[command(alias = "del", alias = "delete")]
    DeleteNetwork(DeleteNetworkArgs),
    /// List cluster VPN networks
    #[command(alias = "l", alias = "list")]
    ListNetworks,
    /// Connect to a network
    #[command(alias = "c")]
    Connect(ConnectArgs),
    /// Disconnect from the network
    #[command(alias = "d")]
    Disconnect,
    /// Get the WireGuard configuration file for a network
    #[command(alias = "g")]
    GetConf(GetConfArgs),
    /// Patch the DNS resolver to avoid loops when deploying on the local machine
    #[command(alias = "p")]
    PatchDns(PatchDnsArgs),
}

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// DNS service IP (autodetected if unset)
    #[arg(long)]
    pub kube_dns: Option<String>,
    /// Cluster service CIDR (autodetected if unset)
    #[arg(long)]
    pub service_cidr: Option<Ipv4Net>,
    /// Cluster domain name assigned to services (autodetected if unset)
    #[arg(long)]
    pub service_domain: Option<String>,
    /// Cluster pod CIDR (autodetected if unset)
    #[arg(long)]
    pub pod_cidr: Option<Ipv4Net>,
    /// don't install CRDs (should you choose not to install them here make sure beforehand they are available on the cluster)
    #[arg(long)]
    pub no_crds: bool,
    /// Substitutes the k8s-insider-controller container image if specified
    #[arg(long, default_value = DEFAULT_CONTROLLER_IMAGE)]
    pub controller_image: String,
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
}

#[derive(Debug, Args)]
pub struct DeleteNetworkArgs {
    /// Name of the network to remove
    #[arg(default_value = DEFAULT_NETWORK_NAME)]
    pub name: String,
    /// If set, no action will be taken on the cluster
    #[arg(long)]
    pub dry_run: bool,
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
pub struct ConnectArgs {
    /// Whether or not to omit patching the DNS resolver on connection
    pub dont_patch_dns: bool,
}

#[derive(Debug, Args)]
pub struct GetConfArgs {
    /// If set, the command will write the config to a file instead of stdout
    #[arg(short = 'o', long)]
    pub output: Option<String>,
}

#[derive(Debug, Args)]
pub struct PatchDnsArgs {
    /// Name of the interface to patch (autodetected if unset)
    pub interface_name: Option<String>,
    /// Cluster domain name assigned to services (autodetected if unset)
    pub services_domain: Option<String>,
}
