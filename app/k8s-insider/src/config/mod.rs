use std::{
    collections::BTreeMap,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::anyhow;

use log::debug;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use self::network::NetworkConfig;

pub mod network;
pub mod tunnel;

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
    networks: BTreeMap<String, NetworkConfig>,
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

    pub fn generate_config_network_name(&self, network: &NetworkConfig) -> String {
        if !self.networks.contains_key(&network.id.name) {
            return network.id.name.to_owned();
        }

        let name_context = format!("{}-{}", network.id.name, network.id.context);
        if !self.networks.contains_key(&name_context) {
            return name_context;
        }

        for index in 0.. {
            let name = format!("{name_context}{index}");

            if !self.networks.contains_key(&name) {
                return name;
            }
        }

        panic!("Your brother's ten times better than you");
    }

    pub fn try_get_default_network(&self) -> anyhow::Result<Option<(&String, &NetworkConfig)>> {
        if self.networks.len() > 1 {
            return Err(anyhow!(
                "No default network: multiple networks written to config!"
            ));
        }

        Ok(self.networks.first_key_value())
    }

    pub fn try_get_network(&self, name: &str) -> Option<(&String, &NetworkConfig)> {
        self.networks.get_key_value(name)
    }

    pub fn try_get_network_mut(&mut self, name: &str) -> Option<&mut NetworkConfig> {
        self.networks.get_mut(name)
    }

    pub fn list_networks(&self) -> impl Iterator<Item = (&String, &NetworkConfig)> {
        self.networks.iter()
    }

    pub fn try_add_network(&mut self, name: String, config: NetworkConfig) -> anyhow::Result<()> {
        if self.networks.contains_key(&name) {
            return Err(anyhow!(
                "Network named {name} is already present in the config!"
            ));
        }

        if self.networks.insert(name, config).is_some() {
            panic!("(*birb*) DONT TOUCH ME (*birb*)");
        }

        Ok(())
    }

    pub fn try_remove_network(&mut self, name: &str) -> anyhow::Result<NetworkConfig> {
        if !self.networks.contains_key(name) {
            return Err(anyhow!("There's no '{name}' tunnel in the config!"));
        }

        Ok(self.networks.remove(name).unwrap())
    }
}
