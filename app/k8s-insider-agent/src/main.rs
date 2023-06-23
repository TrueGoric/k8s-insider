use std::{error::Error, process::exit, sync::Arc};

use futures::StreamExt;
use k8s_insider_core::resources::controller::ControllerRelease;
use kube::{
    runtime::{watcher::Config, Controller},
    Api, Client,
};
use reconciler::{context::ReconcilerContext, reconcile_tunnel, reconcile_tunnel_error, error};

mod reconciler;

#[tokio::main()]
async fn main() -> Result<(), Box<dyn Error>> {
    configure_logger();

    let release = get_release();
    let client = create_client().await;

    let context = ReconcilerContext {
        tunnel_api: Api::namespaced(client.clone(), client.default_namespace()),
        pod_api: Api::namespaced(client.clone(), client.default_namespace()),
        service_api: Api::namespaced(client.clone(), client.default_namespace()),
    };

    Controller::new(context.tunnel_api.clone(), Config::default())
        .owns(context.pod_api.clone(), Config::default())
        .owns(context.service_api.clone(), Config::default())
        .shutdown_on_signal()
        .run(reconcile_tunnel, reconcile_tunnel_error, Arc::new(context))
        .for_each(|_| futures::future::ready(()))
        .await;

    Ok(())
}

async fn create_client() -> Client {
    match Client::try_default().await {
        Ok(client) => client,
        Err(error) => {
            log::error!("Couldn't create client! {error:?}");
            exit(6)
        }
    }
}

fn get_release() -> ControllerRelease {
    match ControllerRelease::from_env() {
        Ok(release) => release,
        Err(error) => {
            log::error!("Couldn't retrieve release info! {error:?}");
            exit(7)
        },
    }
}

fn configure_logger() {
    env_logger::builder()
        .default_format()
        .format_module_path(false)
        .filter_level(log::LevelFilter::Info)
        .init()
}

