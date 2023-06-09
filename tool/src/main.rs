use clap::Parser;
use cli::{Commands, GlobalArgs, LogLevel};
use commands::install::install;
use env_logger::Target;
use log::LevelFilter;

use crate::cli::Cli;

mod cli;
mod commands;
mod detectors;
mod kubernetes;
mod resources;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    configure_logging(&cli.global_args);

    match cli.command {
        Some(command) => match command {
            Commands::Install(args) => install(cli.global_args, args).await?,
            Commands::Uninstall(args) => todo!(),
            Commands::Connect(args) => todo!(),
            Commands::Disconnect => todo!(),
            Commands::GetConf(args) => todo!(),
            Commands::PatchDns(args) => todo!(),
            Commands::Upgrade(args) => todo!(),
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
            _ => false
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
