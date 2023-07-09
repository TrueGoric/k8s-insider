use std::path::Path;

use anyhow::Context;
use clap::Parser;
use cli::{Commands, CreateSubcommands, DeleteSubcommands, GlobalArgs, ListSubcommands, LogLevel};
use commands::{
    create_network::create_network, create_tunnel::create_tunnel, delete_network::delete_network,
    delete_tunnel::delete_tunnel, install::install, list_networks::list_networks,
    uninstall::uninstall,
};
use config::InsiderConfig;
use env_logger::Target;
use k8s_insider_core::kubernetes::operations::create_local_client;
use log::LevelFilter;

use crate::cli::Cli;

mod cli;
mod commands;
mod config;

pub const CLI_FIELD_MANAGER: &str = "k8s-insider";

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    configure_logging(&cli.global_args);

    let client = create_local_client(&cli.global_args.kube_config, &cli.global_args.kube_context)
        .await
        .context("Couldn't initialize k8s API client!")?;
    let config = match cli.global_args.config {
        Some(ref path) => InsiderConfig::load_or_create(Path::new(path)),
        None => InsiderConfig::load_or_create_from_default(),
    }
    .context("Couldn't load k8s-insider configuration file!")?;

    if let Some(command) = cli.command {
        match command {
            Commands::Install(args) => install(cli.global_args, args, client).await?,
            Commands::Uninstall(args) => uninstall(cli.global_args, args, client).await?,
            Commands::Create(create_sub) => match create_sub.subcommand {
                CreateSubcommands::Network(args) => {
                    create_network(cli.global_args, args, client).await?
                }
                CreateSubcommands::Tunnel(args) => {
                    create_tunnel(cli.global_args, args, client, config).await?
                }
            },
            Commands::Delete(delete_sub) => match delete_sub.subcommand {
                DeleteSubcommands::Network(args) => {
                    delete_network(cli.global_args, args, client).await?
                }
                DeleteSubcommands::Tunnel(args) => {
                    delete_tunnel(cli.global_args, args, client, config).await?
                }
            },
            Commands::List(list_sub) => match list_sub.subcommand {
                ListSubcommands::Network => list_networks(cli.global_args, client).await?,
                ListSubcommands::Tunnel => todo!(),
            },
            Commands::Connect(_) => todo!(),
            Commands::Disconnect => todo!(),
            Commands::GetConf(_) => todo!(),
            Commands::PatchDns(_) => todo!(),
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
