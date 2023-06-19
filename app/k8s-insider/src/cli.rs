use clap::{Args, Parser, Subcommand, ValueEnum};

pub const DEAFULT_RELEASE_NAME: &str = "local-access";
pub const DEFAULT_NAMESPACE: &str = "k8s-insider";

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
    /// install k8s-insider on the cluster
    #[command(alias = "i")]
    Install(InstallArgs),
    /// uninstall k8s-insider from the cluster
    #[command(alias = "u")]
    Uninstall(UninstallArgs),
    /// connect to the cluster
    #[command(alias = "c")]
    Connect(ConnectArgs),
    /// disconnect from the cluster
    #[command(alias = "d")]
    Disconnect,
    /// get the WireGuard configuration file from the cluster
    #[command(alias = "g")]
    GetConf(GetConfArgs),
    /// patch the DNS resolver to avoid loops when deploying on the local machine
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
    /// type of the public facing service that will be used to connect to the k8s-insider instance
    #[arg(long, value_enum, default_value_t = ServiceType::NodePort)]
    pub service_type: ServiceType,
    /// manually sets the connection IP (valid for NodePort and ExternalIp service types)
    ///
    /// required for ExternalIp services,
    /// when defined with NodePort service type it skips the autodetection of node IPs and
    /// instructs clients to connect using the provided address.
    #[arg(long)]
    pub external_ip: Option<String>,
    /// DNS service IP (autodetected if unset)
    #[arg(long)]
    pub kube_dns: Option<String>,
    /// cluster service CIDR (autodetected if unset)
    #[arg(long)]
    pub service_cidr: Option<String>,
    /// cluster domain name assigned to services (autodetected if unset)
    #[arg(long)]
    pub service_domain: Option<String>,
    /// cluster pod CIDR (autodetected if unset)
    #[arg(long)]
    pub pod_cidr: Option<String>,
    /// if set, no action will be taken on the cluster
    #[arg(long)]
    pub dry_run: bool,
    /// push the insallation even if it already exists in the cluster
    #[arg(long)]
    pub force: bool,
    /// substitutes the k8s-insider-agent container image if specified
    #[arg(long, default_value = DEFAULT_AGENT_IMAGE)]
    pub agent_image_name: String,
    /// substitutes the k8s-insider-tunnel container image if specified
    #[arg(long, default_value = DEFAULT_TUNNEL_IMAGE)]
    pub tunnel_image_name: String,
}

#[derive(Debug, Args)]
pub struct UninstallArgs {
    /// try to remove the namespace afterwards
    #[arg(long)]
    pub delete_namespace: bool,
    /// if set, no action will be taken on the cluster
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct ConnectArgs {
    /// whether to omit patching the DNS resolver on connection
    pub dont_patch_dns: bool,
}

#[derive(Debug, Args)]
pub struct GetConfArgs {
    /// if set, the command will write the config to a file instead of stdout
    #[arg(short = 'o', long)]
    pub output: Option<String>,
}

#[derive(Debug, Args)]
pub struct PatchDnsArgs {
    /// name of the interface to patch
    pub interface_name: String,
    /// cluster domain name assigned to services
    pub services_domain: String,
}
