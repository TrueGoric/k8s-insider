use std::net::IpAddr;

use clap::{Args, Parser, Subcommand, ValueEnum};
use ipnet::IpNet;

pub const DEFAULT_NAMESPACE: &str = "k8s-insider";

pub const DEFAULT_PEER_CIDR: &str = "10.11.11.0/24";

pub const DEFAULT_TUNNEL_IMAGE: &str = "ghcr.io/truegoric/k8s-insider-tunnel:latest";
pub const DEFAULT_AGENT_IMAGE: &str = "ghcr.io/truegoric/k8s-insider-agent:latest";

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
    /// Connect to the cluster
    #[command(alias = "c")]
    Connect(ConnectArgs),
    /// Disconnect from the cluster
    #[command(alias = "d")]
    Disconnect,
    /// Get the WireGuard configuration file from the cluster
    #[command(alias = "g")]
    GetConf(GetConfArgs),
    /// Patch the DNS resolver to avoid loops when deploying on the local machine
    #[command(alias = "p")]
    PatchDns(PatchDnsArgs),
}

#[derive(Debug, Clone, ValueEnum)]
#[value()]
pub enum ServiceType {
    #[value(name = "None")]
    None,
    #[value(name = "NodePort")]
    NodePort,
    #[value(name = "LoadBalancer")]
    LoadBalancer,
    #[value(name = "ExternalIp")]
    ExternalIp,
}

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Type of the public facing service that will be used to connect to the k8s-insider instance
    #[arg(long, value_enum, default_value_t = ServiceType::NodePort)]
    pub service_type: ServiceType,
    /// Manually sets the connection IP (valid for NodePort and ExternalIp service types)
    ///
    /// Required for ExternalIp services,
    /// When defined with NodePort service type it skips the autodetection of node IPs and
    /// instructs clients to connect using the provided address.
    #[arg(long)]
    pub external_ip: Option<String>,
    /// DNS service IP (autodetected if unset)
    #[arg(long)]
    pub kube_dns: Option<String>,
    /// Cluster service CIDR (autodetected if unset)
    #[arg(long)]
    pub service_cidr: Option<IpNet>,
    /// Cluster domain name assigned to services (autodetected if unset)
    #[arg(long)]
    pub service_domain: Option<String>,
    /// Cluster pod CIDR (autodetected if unset)
    #[arg(long)]
    pub pod_cidr: Option<IpNet>,
    /// Peer CIDR subnet (users will be assigned IPs from that range)
    #[arg(long, default_value = DEFAULT_PEER_CIDR)]
    pub peer_cidr: IpNet,
    /// Router's IP address, must be within peer CIDR range (defaults to the first non-broadcast IP from that range)
    #[arg(long)]
    pub router_ip: Option<IpAddr>,    
    /// If set, no action will be taken on the cluster
    #[arg(long)]
    pub dry_run: bool,
    /// Push the insallation even if it already exists in the cluster
    #[arg(long)]
    pub force: bool,
    /// Substitutes the k8s-insider-agent container image if specified
    #[arg(long, default_value = DEFAULT_AGENT_IMAGE)]
    pub agent_image_name: String,
    /// Substitutes the k8s-insider-tunnel container image if specified
    #[arg(long, default_value = DEFAULT_TUNNEL_IMAGE)]
    pub tunnel_image_name: String,
}

#[derive(Debug, Args)]
pub struct UninstallArgs {
    /// Try to remove the namespace afterwards
    #[arg(long)]
    pub delete_namespace: bool,
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
