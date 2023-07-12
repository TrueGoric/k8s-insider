use futures::{Future, StreamExt};
use k8s_insider_core::{kubernetes::GetApi, resources::crd::v1alpha1::tunnel::Tunnel};
use kube::runtime::{
    reflector::{self, reflector, Store},
    watcher::{watcher, Config},
};
use tokio::sync::watch::{self, Receiver};

use super::reconciler::context::ReconcilerContext;

pub fn start_tunnel_reflector(
    context: &ReconcilerContext,
) -> (impl Future<Output = ()>, Store<Tunnel>, Receiver<()>) {
    let (tx, rx) = watch::channel::<()>(());

    let watcher_config = Config::default();
    let watcher = watcher(
        context
            .client
            .namespaced_api::<Tunnel>(&context.router_info.namespace),
        watcher_config,
    );

    let (store, writer) = reflector::store();
    let reflector = reflector(writer, watcher).for_each(move |_| {
        tx.send(()).unwrap();
        std::future::ready(())
    });

    (reflector, store, rx)
}
