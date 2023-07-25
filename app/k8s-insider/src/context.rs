use std::path::Path;

use anyhow::{anyhow, Context};
use k8s_insider_core::helpers::With;
use kube::{
    config::{KubeConfigOptions, Kubeconfig},
    Client, Config,
};

use crate::{config::InsiderConfig, wireguard::connection_manager::ConnectionManager};

pub const DEFAULT_CONFIG_FILENAME: &str = "insider-config";
pub const KUBECONFIG_ENV_VAR: &str = "KUBECONFIG";
pub const WGCONF_RELATIVE_FOLDER: &str = "insider-tunnels";

pub struct ConfigContext {
    pub kube_context_name: String,

    pub kube_config: Kubeconfig,
    pub insider_config: InsiderConfig,
    pub connections: ConnectionManager,
}

impl ConfigContext {
    pub fn new(
        kube_config_path: Option<&Path>,
        insider_config_path: Option<&Path>,
        kube_context: Option<&str>,
    ) -> anyhow::Result<Self> {
        let kube_config_path = match kube_config_path {
            Some(path) => path.to_owned(),
            None => std::env::var(KUBECONFIG_ENV_VAR)
                .map(|s| s.into())
                .or_else(|_| {
                    home::home_dir()
                        .map(|d| d.with(|d| d.push(".kube")).with(|d| d.push("config")))
                        .ok_or(anyhow!("Missing home dir!"))
                })?,
        };

        let kube_config =
            Kubeconfig::read_from(&kube_config_path).context("Couldn't load kubeconfig!")?;
        let insider_config_path = match insider_config_path {
            Some(path) => {
                if path.file_name().is_none() {
                    return Err(anyhow!("Insider config path is invalid!"));
                }

                path.into()
            }
            None => kube_config_path
                .as_path()
                .parent()
                .unwrap_or(Path::new(""))
                .to_owned()
                .with(|d| d.push(DEFAULT_CONFIG_FILENAME)),
        };
        let insider_config = InsiderConfig::load_or_create(&insider_config_path)?;
        let kube_context_name = match kube_context {
            Some(name) => name.to_owned(),
            None => kube_config
                .current_context
                .as_ref()
                .ok_or(anyhow!(
                    "Current context isn't set! Desired context must be specified manually!"
                ))?
                .to_owned(),
        };

        let wgconf_path = insider_config_path
            .parent()
            .unwrap()
            .join(WGCONF_RELATIVE_FOLDER);
        let connections = ConnectionManager::load(wgconf_path)?;

        Ok(Self {
            kube_config,
            insider_config,
            connections,
            kube_context_name,
        })
    }

    pub fn kube_context_name(&self) -> &str {
        &self.kube_context_name
    }

    pub fn insider_config(&self) -> &InsiderConfig {
        &self.insider_config
    }

    pub fn insider_config_mut(&mut self) -> &mut InsiderConfig {
        &mut self.insider_config
    }

    pub async fn create_client_with_default_context(&self) -> anyhow::Result<Client> {
        self.create_client(&self.kube_context_name).await
    }

    pub async fn create_client(&self, context: &str) -> anyhow::Result<Client> {
        let config_options = KubeConfigOptions {
            context: Some(context.to_owned()),
            ..Default::default()
        };

        let config =
            Config::from_custom_kubeconfig(self.kube_config.clone(), &config_options).await?;
        let client = Client::try_from(config)?;

        Ok(client)
    }
}
