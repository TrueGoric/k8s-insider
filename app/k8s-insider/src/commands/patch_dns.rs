use crate::{cli::PatchDnsArgs, context::ConfigContext};

pub async fn patch_dns(args: PatchDnsArgs, mut context: ConfigContext) -> anyhow::Result<()> {
    let (_, network_config) = context
        .insider_config
        .get_network_or_default(args.network.as_deref())?;

    context.connections.patch_dns(&network_config.id)?;

    Ok(())
}
