use anyhow::Context;
use k8s_insider_core::wireguard::keys::WgKey;

use crate::{
    cli::{ConfigAddTunnelArgs, GlobalArgs},
    config::{tunnel::TunnelConfig, ConfigContext},
};

pub fn config_add_tunnel(
    global_args: GlobalArgs,
    args: ConfigAddTunnelArgs,
    mut context: ConfigContext,
) -> anyhow::Result<()> {
    eprint!("Enter the private key: ");
    let private_key = WgKey::from_base64_stdin()
        .context("Invalid private key from stdin!")?
        .to_base64();
    let kube_context = global_args
        .kube_context
        .unwrap_or_else(|| context.kube_context_name().to_owned());

    let tunnel = TunnelConfig {
        context: kube_context,
        name: args.name,
        namespace: global_args.namespace,
        private_key,
    };

    let config = context.insider_config_mut();
    let local_name = args
        .local_name
        .unwrap_or_else(|| config.generate_config_tunnel_name(&args.network));

    if config.tunnels.insert(local_name, tunnel).is_some() {
        panic!("Tunnel name collision - please report this error if you encounter it");
    }

    config.save()?;

    Ok(())
}
