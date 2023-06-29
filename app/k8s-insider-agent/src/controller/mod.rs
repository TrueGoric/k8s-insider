use std::{process::exit, sync::Arc};

use k8s_insider_core::resources::controller::ControllerRelease;
use kube::Client;

use crate::controller::reconciler::context::ReconcilerContext;

use self::network::start_network_controller;

pub mod network;
pub mod reconciler;

pub const CONTROLLER_FIELD_MANAGER: &str = "k8s-insider-controller";

pub async fn main_controller(client: Client) {
    let reconciler_context = Arc::new(get_reconciler_context(client));

    start_network_controller(&reconciler_context).await;
}

fn get_reconciler_context(client: Client) -> ReconcilerContext {
    ReconcilerContext {
        release: get_controller_release(),
        client,
    }
}

fn get_controller_release() -> ControllerRelease {
    match ControllerRelease::from_env() {
        Ok(release) => release,
        Err(error) => {
            log::error!("Couldn't retrieve release info! {error:?}");
            exit(7)
        }
    }
}
