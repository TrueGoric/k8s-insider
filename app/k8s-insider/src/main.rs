use std::path::Path;

use clap::Parser;
use cli::{
    Commands, ConfigSubcommands, CreateSubcommands, DeleteSubcommands, GlobalArgs, ListSubcommands,
    LogLevel,
};
use commands::{
    config_add_tunnel::config_add_tunnel, config_list_tunnels::config_list_tunnels,
    config_remove_tunnel::config_remove_tunnel, connect::connect, create_network::create_network,
    create_tunnel::create_tunnel, delete_network::delete_network, delete_tunnel::delete_tunnel,
    get_configuration::get_configuration, install::install, list_networks::list_networks,
    uninstall::uninstall,
};
use config::ConfigContext;
use env_logger::Target;
use log::LevelFilter;

use crate::cli::Cli;

mod cli;
mod commands;
mod config;
mod output;
mod wireguard;

pub const CLI_FIELD_MANAGER: &str = "k8s-insider";

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    configure_logging(&cli.global_args);

    let mut context = ConfigContext::new(
        cli.global_args.kube_config.as_deref().map(Path::new),
        cli.global_args.config.as_deref().map(Path::new),
        cli.global_args.kube_context.as_deref(),
    )?;

    if let Some(command) = cli.command {
        match command {
            Commands::Install(args) => install(cli.global_args, args, context).await?,
            Commands::Uninstall(args) => uninstall(cli.global_args, args, context).await?,
            Commands::Create(create_sub) => match create_sub.subcommand {
                CreateSubcommands::Network(args) => {
                    create_network(cli.global_args, args, context).await?
                }
                CreateSubcommands::Tunnel(args) => {
                    create_tunnel(&cli.global_args, &args, &mut context).await?
                }
            },
            Commands::Delete(delete_sub) => match delete_sub.subcommand {
                DeleteSubcommands::Network(args) => {
                    delete_network(cli.global_args, args, context).await?
                }
                DeleteSubcommands::Tunnel(args) => {
                    delete_tunnel(cli.global_args, args, context).await?
                }
            },
            Commands::List(list_sub) => match list_sub.subcommand {
                ListSubcommands::Network(args) => list_networks(cli.global_args, args, context).await?,
                ListSubcommands::Tunnel(_) => todo!(),
            },
            Commands::Connect(args) => connect(cli.global_args, args, context).await?,
            Commands::Disconnect => todo!(),
            Commands::GetConf(args) => get_configuration(args, context).await?,
            Commands::PatchDns(_) => todo!(),
            Commands::Config(config_sub) => match config_sub.subcommand {
                ConfigSubcommands::AddTunnel(args) => {
                    config_add_tunnel(cli.global_args, args, context)?
                }
                ConfigSubcommands::ListTunnels(args) => config_list_tunnels(args, context)?,
                ConfigSubcommands::RemoveTunnel(args) => config_remove_tunnel(args, context)?,
            },
        }
    }

    Ok(())
}

fn configure_logging(global_args: &GlobalArgs) {
    let log_level = global_args.get_log_level();
    let mut logger = env_logger::builder();

    logger
        .format_timestamp(None)
        .format_module_path(matches!(log_level, LogLevel::Trace))
        .format_target(false)
        .format_level(false)
        .target(Target::Stderr);

    if let LogLevel::Normal = log_level {
        logger.filter(Some("k8s_insider"), LevelFilter::Info);
    }

    if let LogLevel::Verbose = log_level {
        logger.filter(Some("k8s_insider"), LevelFilter::Debug);
    }

    if let LogLevel::Trace = log_level {
        logger.filter(None, LevelFilter::Debug);
    }

    logger.init();
}
