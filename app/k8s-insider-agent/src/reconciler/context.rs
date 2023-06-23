use k8s_insider_core::resources::crd::v1alpha1::tunnel::Tunnel;
use k8s_openapi::api::core::v1::{Pod, Service};
use kube::Api;

pub struct ReconcilerContext {
    pub tunnel_api: Api<Tunnel>,
    pub pod_api: Api<Pod>,
    pub service_api: Api<Service>,
}

// .server_private_key({
//     info!("Using generated private server key!");
//     generate_wireguard_private_key()
// })
// .peer_cidr({
//     info!("Using peer CIDR: {}", args.peer_cidr);
//     args.peer_cidr.trunc()
// })
// .router_ip({
//     let ip = match &args.router_ip {
//         Some(value) => value.to_owned(),
//         None => args.peer_cidr.hosts().next().unwrap().clone(),
//     };
//     info!("Using router IP: {ip}");
//     ip
// })
// .service(match &args.service_type {
//     ServiceType::None => RouterReleaseService::None,
//     ServiceType::NodePort => RouterReleaseService::NodePort {
//         predefined_ips: args.external_ip.clone()
//     },
//     ServiceType::LoadBalancer => RouterReleaseService::LoadBalancer,
//     ServiceType::ExternalIp => RouterReleaseService::ExternalIp {
//         ip: args.external_ip
//             .as_ref()
//             .ok_or_else(|| anyhow!("--external-ip argument is mandatory when using service of type ExternalIp!"))?
//             .clone()
//         },
// })