use k8s_insider_core::{
    kubernetes::operations::try_get_resource,
    resources::controller::{ControllerRelease, CONTROLLER_RELEASE_NAME},
};
use k8s_insider_macros::TableOutputRow;
use k8s_openapi::api::core::v1::ConfigMap;

use log::debug;
use serde::Serialize;

use crate::{
    cli::{GlobalArgs, VersionArgs},
    context::ConfigContext,
    output::CliPrint,
    version::{get_latest_version, LOCAL_INSIDER_VERSION},
};

pub async fn print_version(
    global_args: GlobalArgs,
    args: VersionArgs,
    context: ConfigContext,
) -> anyhow::Result<()> {
    let client = context.create_client_with_default_context().await?;
    let release_configmap =
        try_get_resource::<ConfigMap>(&client, CONTROLLER_RELEASE_NAME, &global_args.namespace)
            .await;

    let release = match release_configmap {
        Ok(ref configmap) => match configmap {
            Some(configmap) => match ControllerRelease::from_configmap(configmap) {
                Ok(release) => Some(release),
                Err(error) => {
                    debug!("Couldn't parse release ConfigMap! {error}");
                    None
                }
            },
            None => {
                debug!("Couldn't fetch release ConfigMap! Release not found!");
                None
            }
        },
        Err(ref error) => {
            debug!("Couldn't fetch release ConfigMap! {error}");
            None
        }
    };

    let latest_version = match get_latest_version().await {
        Ok(version) => Some(version),
        Err(error) => {
            debug!("Couldn't fetch latest k8s-insider version! {error}");

            None
        }
    };

    let version_table = vec![
        VersionView::get_local_cli_version(),
        VersionView::get_latest_version(latest_version.as_deref()),
        VersionView::get_controller_image_version(release.as_ref()),
        VersionView::get_network_manager_image_version(release.as_ref()),
        VersionView::get_router_image_version(release.as_ref()),
    ];

    version_table.print(args.output)?;

    Ok(())
}

#[derive(Serialize, TableOutputRow)]
struct VersionView<'a> {
    #[name_column]
    pub component: &'a str,
    pub version: &'a str,
}

impl<'a> VersionView<'a> {
    pub fn get_local_cli_version() -> Self {
        Self {
            component: "k8s-insider CLI (local)",
            version: LOCAL_INSIDER_VERSION,
        }
    }

    pub fn get_latest_version(version: Option<&'a str>) -> Self {
        Self {
            component: "k8s-insider CLI (latest)",
            version: version.unwrap_or("unknown"),
        }
    }

    pub fn get_controller_image_version(release: Option<&'a ControllerRelease>) -> Self {
        Self {
            component: "k8s-insider-controller",
            version: release
                .map(|r| r.controller_image_tag.as_str())
                .unwrap_or("unknown"),
        }
    }

    pub fn get_network_manager_image_version(release: Option<&'a ControllerRelease>) -> Self {
        Self {
            component: "k8s-insider-network-manager",
            version: release
                .map(|r: &ControllerRelease| r.network_manager_image_tag.as_str())
                .unwrap_or("unknown"),
        }
    }

    pub fn get_router_image_version(release: Option<&'a ControllerRelease>) -> Self {
        Self {
            component: "k8s-insider-router",
            version: release
                .map(|r| r.router_image_tag.as_str())
                .unwrap_or("unknown"),
        }
    }
}
