use std::{error::Error, process::exit};

use kube::Client;
use log::{error, info, LevelFilter};

use crate::{controller::main_controller, network_manager::main_network_manager};

mod controller;
mod helpers;
mod network_manager;
mod release;

#[tokio::main()]
async fn main() -> Result<(), Box<dyn Error>> {
    configure_logger();

    let client = create_client().await;
    let args = std::env::args().collect::<Vec<String>>();
    let mode = match args.get(1) {
        Some(val) => val.as_str(),
        None => {
            error!("Missing deployment mode (should be controller or router)!");
            exit(1)
        }
    };

    match mode {
        "controller" => {
            info!("Starting agent in controller mode...");
            main_controller(client).await
        }
        "network-manager" => {
            info!("Starting agent in network-manager mode...");
            main_network_manager(client).await
        }
        "router" => {
            info!("Starting agent in router mode...");
            todo!()
        }
        _ => {
            error!("Unsupported deployment mode!");
            exit(1)
        }
    };

    info!("Exiting...");

    Ok(())
}

async fn create_client() -> Client {
    match Client::try_default().await {
        Ok(client) => client,
        Err(error) => {
            error!("Couldn't create the client! {error:?}");
            exit(6)
        }
    }
}

fn configure_logger() {
    env_logger::builder()
        .default_format()
        .format_module_path(false)
        .filter_level(LevelFilter::Info)
        .filter(
            Some("k8s_insider_core::kubernetes::operations"),
            LevelFilter::Warn,
        )
        .init()
}
