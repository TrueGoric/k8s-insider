use anyhow::{anyhow, Context};
use k8s_insider_core::{
    helpers::{RequireMetadata, With},
    ip::addrpair::IpAddrPair,
    kubernetes::operations::create_resource,
    resources::crd::v1alpha1::tunnel::{Tunnel, TunnelSpec},
    wireguard::keys::WgKey,
};
use kube::{api::PostParams, core::ObjectMeta};
use log::{debug, info};

use crate::{
    cli::{CreateTunnelArgs, GlobalArgs},
    config::{tunnel::TunnelConfig, ConfigContext},
    CLI_FIELD_MANAGER,
};

pub async fn create_tunnel(
    global_args: &GlobalArgs,
    args: &CreateTunnelArgs,
    context: &mut ConfigContext,
) -> anyhow::Result<()> {
    let config_network = context
        .insider_config()
        .try_get_network(&args.network)
        .ok_or(anyhow!(
            "Couldn't find network '{}' in the config!",
            args.network
        ))?;
    let client = context.create_client(&config_network.1.id.context).await?;

    info!(
        "Creating a tunnel for '{}' network in '{}' namespace...",
        args.network, global_args.namespace
    );

    let private_key = WgKey::generate_private_key();
    let public_key = private_key.get_public();
    let preshared_key = WgKey::generate_preshared_key();

    let apply_params =
        PostParams::default().with(|p| p.field_manager = Some(CLI_FIELD_MANAGER.to_owned()));
    let tunnel_crd = create_tunnel_crd(
        &config_network.1.id.name,
        &config_network.1.id.namespace,
        public_key,
        preshared_key,
        args.static_ip.map(|ipv4| ipv4.into()),
    );

    debug!("{tunnel_crd:#?}");

    create_resource(&client, &tunnel_crd, &apply_params).await?;
    write_config(
        args.name.as_deref(),
        &args.network,
        &tunnel_crd,
        private_key,
        context,
    )?;

    info!("Tunnel successfully created!");

    Ok(())
}

fn create_tunnel_crd(
    network_name: &str,
    namespace: &str,
    public_key: WgKey,
    preshared_key: WgKey,
    static_ip: Option<IpAddrPair>,
) -> Tunnel {
    // CRD resource names must be valid DNS subdomains, so Base64 is out of the question
    // this public key representation conforms to https://datatracker.ietf.org/doc/html/rfc5155
    let tunnel_name = format!("{network_name}-{}", public_key.to_dnssec_base32());

    Tunnel {
        metadata: ObjectMeta {
            name: Some(tunnel_name),
            namespace: Some(namespace.to_owned()),
            ..Default::default()
        },
        spec: TunnelSpec {
            network: network_name.to_owned(),
            peer_public_key: public_key.to_base64(),
            preshared_key: preshared_key.to_base64(),
            static_ip,
        },
        status: None,
    }
}

fn write_config(
    name: Option<&str>,
    network: &str,
    crd: &Tunnel,
    private_key: WgKey,
    context: &mut ConfigContext,
) -> anyhow::Result<()> {
    let config_network = context
        .insider_config_mut()
        .try_get_network_mut(network)
        .ok_or(anyhow!(
            "Couldn't find network '{}' in the config!",
            network
        ))?;

    let entry = TunnelConfig::new(
        crd.require_name_or(anyhow!("Missing Tunnel CRD name!"))?
            .to_owned(),
        private_key,
    );

    let local_name = name
        .map(|s| s.to_owned())
        .unwrap_or_else(|| config_network.generate_config_tunnel_name());

    config_network.try_add_tunnel(local_name, entry)?;

    context
        .insider_config()
        .save()
        .context("Couldn't save the configuration file!")?;

    Ok(())
}
