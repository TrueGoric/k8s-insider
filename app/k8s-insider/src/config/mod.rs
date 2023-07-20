use std::{
    collections::BTreeMap,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use k8s_insider_core::helpers::With;
use kube::{
    config::{KubeConfigOptions, Kubeconfig},
    Client, Config,
};
use log::debug;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use self::tunnel::TunnelConfig;

pub mod tunnel;

pub const DEFAULT_CONFIG_FILENAME: &str = "insider-config";
pub const KUBECONFIG_ENV_VAR: &str = "KUBECONFIG";

#[derive(Debug, Error)]
pub enum InsiderConfigError {
    #[error("Io error: {}", .0)]
    IoError(std::io::Error),
    #[error("Serialization error: {}", .0)]
    SerializationError(serde_yaml::Error),
    #[error("Deserialization error: {}", .0)]
    DeserializationError(serde_yaml::Error),
    #[error("Configuration path is unspecified!")]
    ConfigPathUnspecified,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct InsiderConfig {
    #[serde(skip)]
    pub path: Option<PathBuf>,
    pub tunnels: BTreeMap<String, TunnelConfig>,
}

impl InsiderConfig {
    pub fn load_or_create(path: &Path) -> Result<Self, InsiderConfigError> {
        debug!("Used config path: {path:?}");

        let file = File::options()
            .create(true)
            .write(true)
            .read(true)
            .open(path)
            .map_err(InsiderConfigError::IoError)?;

        let mut config: InsiderConfig =
            serde_yaml::from_reader(file).map_err(InsiderConfigError::DeserializationError)?;

        config.path = Some(path.to_owned());

        Ok(config)
    }

    pub fn save(&self) -> Result<(), InsiderConfigError> {
        let path = self
            .path
            .as_ref()
            .ok_or(InsiderConfigError::ConfigPathUnspecified)?;
        let file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(InsiderConfigError::IoError)?;

        serde_yaml::to_writer(file, self).map_err(InsiderConfigError::SerializationError)?;

        Ok(())
    }

    pub fn try_get_default_tunnel(&self) -> anyhow::Result<Option<(&String, &TunnelConfig)>> {
        if self.tunnels.len() > 1 {
            return Err(anyhow!(
                "No default tunnel: multiple tunnels written to config!"
            ));
        }

        Ok(self.tunnels.first_key_value())
    }

    pub fn try_get_tunnel(&self, name: &str) -> Option<(&String, &TunnelConfig)> {
        self.tunnels.get_key_value(name)
    }

    pub fn generate_config_tunnel_name(&self, network_name: &str) -> String {
        for index in 0.. {
            let name = format!("{network_name}-tun{index}");

            if !self.tunnels.contains_key(&name) {
                return name;
            }
        }

        panic!("You disobeyed my orders son, why were you ever born?");
    }
}

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
                    return Err(anyhow!("Insider config path is invalid!"))
                }

                path.into()
            },
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

    pub fn kube_config_path(&self) -> &Path {
        &self.kube_config_path
    }

    pub fn kube_config(&self) -> &Kubeconfig {
        &self.kube_config
    }

    pub fn kube_config_mut(&mut self) -> &mut Kubeconfig {
        &mut self.kube_config
    }

    pub fn kube_context_name(&self) -> &str {
        &self.kube_context_name
    }

    pub fn insider_config_path(&self) -> &Path {
        &self.insider_config_path
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
