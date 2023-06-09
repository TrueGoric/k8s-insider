use clap::{Parser, Subcommand, Args};

pub const DEAFULT_RELEASE_NAME: &str = "local-access";
pub const DEFAULT_NAMESPACE: &str = "k8s-insider";

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    #[command(flatten)]
    pub global_args: GlobalArgs
}

#[derive(Debug, Args)]
pub struct GlobalArgs {
    /// kubernetes namespace to work with
    #[arg(short = 'n', long, global = true, default_value = DEFAULT_NAMESPACE)]
    pub namespace: String,
    /// override default kubeconfig
    #[arg(long, global = true)]
    pub kube_config: Option<String>,
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
    Trace
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
    /// check for a new verion of the tool and upgrade
    #[command(alias = "update")]
    Upgrade(UpgradeArgs)
}

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// name of the release (must be unique within the namespace)
    #[arg(default_value = DEAFULT_RELEASE_NAME)]
    pub release_name: String,
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
}

#[derive(Debug, Args)]
pub struct UninstallArgs {
    /// name of the release to uninstall (required if there's more than one configured)
    #[arg()]
    pub release_name: Option<String>,
}

#[derive(Debug, Args)]
pub struct ConnectArgs {
    /// name of the release to connect to (required if there's more than one configured)
    #[arg()]
    pub release_name: Option<String>,
    /// whether to omit patching the DNS resolver on connection
    pub dont_patch_dns: bool,
}

#[derive(Debug, Args)]
pub struct GetConfArgs {
    /// name of the release to connect to (required if there's more than one configured)
    #[arg()]
    pub release_name: Option<String>,
    /// if set, the command will write the config to a file instead of stdout
    #[arg(short = 'o', long)]
    pub output: Option<String>,
}

#[derive(Debug, Args)]
pub struct PatchDnsArgs {
    /// name of the interface to patch
    pub interface_name: String,
    /// cluster domain name assigned to services
    pub services_domain: String
}

#[derive(Debug, Args)]
pub struct UpgradeArgs {
    /// just check for new version, don't perfom the update
    #[arg(short = 'c', long)]
    pub check_only: bool,
}