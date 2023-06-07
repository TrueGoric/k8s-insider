use clap::Parser;
use cli::Commands;

use crate::cli::Cli;

mod cli;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(command) => match command {
            Commands::Install(install_args) => todo!(),
            Commands::Uninstall(uninstall_args) => todo!(),
            Commands::Connect(connect_args) => todo!(),
            Commands::Disconnect => todo!(),
            Commands::GetConf(get_conf_args) => todo!(),
            Commands::PatchDns(patch_dns_args) => todo!(),
            Commands::Upgrade(upgrade_args) => todo!(),
        },
        None => return,
    }
}
