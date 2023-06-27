use anyhow::Context;
use clap::Parser;
use cli::{Commands, GlobalArgs, LogLevel};
use commands::{create_network::create_network, install::install, uninstall::uninstall};
use env_logger::Target;
use k8s_insider_core::kubernetes::operations::create_local_client;
use log::LevelFilter;

use crate::cli::Cli;

mod cli;
mod commands;
mod operations;

pub const CLI_FIELD_MANAGER: &str = "k8s-insider";

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    configure_logging(&cli.global_args);

    let client = create_local_client(&cli.global_args.kube_config, &cli.global_args.kube_context)
        .await
        .context("Couldn't initialize k8s API client!")?;

    if let Some(command) = cli.command {
        match command {
            Commands::Install(args) => install(cli.global_args, args, client).await?,
            Commands::Uninstall(args) => uninstall(cli.global_args, args, client).await?,
            Commands::CreateNetwork(args) => create_network(cli.global_args, args, client).await?,
            Commands::DeleteNetwork => todo!(),
            Commands::ListNetworks => todo!(),
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
