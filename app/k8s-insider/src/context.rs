use std::path::{PathBuf, Path};

use anyhow::{anyhow, Context};
use k8s_insider_core::helpers::With;
use kube::{config::{Kubeconfig, KubeConfigOptions}, Client, Config};

use crate::config::InsiderConfig;

pub const DEFAULT_CONFIG_FILENAME: &str = "insider-config";
pub const KUBECONFIG_ENV_VAR: &str = "KUBECONFIG";

pub struct ConfigContext {
    kube_config_path: PathBuf,
    insider_config_path: PathBuf,
    kube_config: Kubeconfig,
    insider_config: InsiderConfig,
    kube_context_name: String,
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

        Ok(Self {
            kube_config_path,
            insider_config_path,
            kube_config,
            insider_config,
            kube_context_name,
        })
    }

    pub fn kube_config(&self) -> &Kubeconfig {
        &self.kube_config
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

    pub fn get_path_base(&self) -> &Path {
        self.insider_config_path.parent().unwrap() // unwrapping is safe since we make sure that this path contains at least a filename
    }
}
