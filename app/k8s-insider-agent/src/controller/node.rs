use futures::{Future, StreamExt, TryStreamExt};
use k8s_insider_core::kubernetes::GetApi;
use k8s_openapi::api::core::v1::{Node, NodeStatus};
use kube::{
    runtime::{
        reflector::{self, reflector, Store},
        watcher::{watcher, Config},
        WatchStreamExt,
    },
    Client, ResourceExt,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

pub async fn start_node_reflector(
    client: &Client,
) -> (impl Future<Output = ()>, Store<Node>, UnboundedReceiver<()>) {
    let (tx, rx) = unbounded_channel::<()>();
    let watcher_config = Config::default();
    let watcher = watcher(client.global_api::<Node>(), watcher_config).map_ok(|event| {
        event.modify(|node| {
            node.managed_fields_mut().clear();
            node.annotations_mut().clear();
            node.labels_mut().clear();
            node.finalizers_mut().clear();
            node.managed_fields_mut().clear();
            node.owner_references_mut().clear();
            node.spec = None;

            let addresses = node
                .status
                .as_ref()
                .and_then(|status| status.addresses.to_owned());

            node.status = Some(NodeStatus {
                addresses,
                ..Default::default()
            })
        })
    });
    let (store, writer) = reflector::store();
    let reflector = reflector(writer, watcher)
        .applied_objects()
        .for_each(move |_| {
            tx.send(()).unwrap();
            std::future::ready(())
        });

    (reflector, store, rx)
}
