use std::process::exit;

use kube::Client;
use log::{error, info};

use crate::{
    release::{get_ready_network_crd, get_router_info_with_secret},
    router::{tunnel::start_tunnel_reflector, wg_config::ConfigurationSynchronizer},
};

use self::reconciler::context::ReconcilerContext;

pub mod reconciler;
pub mod tunnel;
pub mod wg_config;

pub const _ROUTER_FIELD_MANAGER: &str = "k8s-insider-router";

pub const WIREGUARD_CONFIG_DIRECTORY: &str = "/config";
pub const WIREGUARD_CONFIG_PATH: &str = "/config/wg0.conf";

pub async fn main_router(client: Client) {
    let network_crd = get_ready_network_crd(&client).await;
    let router_info = get_router_info_with_secret(&network_crd);

    let reconciler_context = ReconcilerContext {
        router_info,
        owner: network_crd,
        client,
    };

    let (tunnel_reflector, store, rx) = start_tunnel_reflector(&reconciler_context);
    let mut config_sync = ConfigurationSynchronizer::new(reconciler_context, store, rx);

    let reflector_job = tokio::spawn(tunnel_reflector);
    let sync_job = tokio::spawn(async move { config_sync.start().await });

    reflector_job.await.unwrap();
    sync_job.await.unwrap();
}

pub async fn main_router_config_gen(client: Client) {
    let network_crd = get_ready_network_crd(&client).await;
    let server_config = get_router_info_with_secret(&network_crd)
        .generate_server_wg_config()
        .unwrap_or_else(|err| {
            error!("Couldn't generate WireGuard server configuration! {err:#?}");
            exit(100)
        });

    tokio::fs::create_dir_all(WIREGUARD_CONFIG_DIRECTORY)
        .await
        .unwrap_or_else(|err| {
            error!("Couldn't create WireGuard server configuration directory! {err:#?}");
            exit(101)
        });

    tokio::fs::write(WIREGUARD_CONFIG_PATH, server_config)
        .await
        .unwrap_or_else(|err| {
            error!("Couldn't write WireGuard server configuration! {err:#?}");
            exit(102)
        });

    info!("Configuration written to {WIREGUARD_CONFIG_PATH}!");
}
