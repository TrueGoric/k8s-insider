use anyhow::Context;
use clap::Parser;
use cli::{Commands, GlobalArgs, LogLevel};
use commands::{
    install::install,
    list::list,
    uninstall::{uninstall, uninstall_all}, connect::connect,
};
use env_logger::Target;
use log::LevelFilter;
use operations::kubernetes::create_client;

use crate::cli::Cli;

mod cli;
mod commands;
mod detectors;
mod helpers;
mod operations;
mod resources;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    configure_logging(&cli.global_args);

    let client = create_client(&cli.global_args.kube_config, &cli.global_args.kube_context)
        .await
        .context("Couldn't initialize k8s API client!")?;

    match &cli.command {
        Some(command) => match command {
            Commands::Install(args) => install(&cli.global_args, args, &client).await?,
            Commands::Uninstall(args) => uninstall(&cli.global_args, args, &client).await?,
            Commands::UninstallAll(args) => uninstall_all(&cli.global_args, args, &client).await?,
            Commands::List => list(&cli.global_args, &client).await?,
            Commands::Connect(args) => connect(&cli.global_args, args, &client).await?,
            Commands::Disconnect => todo!(),
            Commands::GetConf(args) => todo!(),
            Commands::PatchDns(args) => todo!(),
        },
        None => (),
    }

    Ok(())
}

fn configure_logging(global_args: &GlobalArgs) {
    let log_level = global_args.get_log_level();
    let mut logger = env_logger::builder();

    logger
        .format_timestamp(None)
        .format_module_path(match log_level {
            LogLevel::Trace => true,
            _ => false,
        })
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
