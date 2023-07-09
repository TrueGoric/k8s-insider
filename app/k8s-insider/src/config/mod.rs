use std::{
    collections::BTreeMap,
    fs::File,
    path::{Path, PathBuf},
};

use log::debug;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use self::tunnel::TunnelConfig;

pub mod tunnel;

pub const DEFAULT_CONFIG_FILENAME: &str = "insider-config";

#[derive(Debug, Error)]
pub enum InsiderConfigError {
    #[error("Couldn't establish user's home directory!")]
    UnknownHomeDir,
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

    pub fn load_or_create_from_default() -> Result<Self, InsiderConfigError> {
        Self::load_or_create(&Self::get_default_path()?)
    }

    fn get_default_path() -> Result<PathBuf, InsiderConfigError> {
        if let Ok(kubeconfig_path) = std::env::var("KUBECONFIG") {
            let mut path: PathBuf = kubeconfig_path.into();

            path.pop();
            path.push(DEFAULT_CONFIG_FILENAME);

            return Ok(path);
        }

        let mut path = home::home_dir().ok_or(InsiderConfigError::UnknownHomeDir)?;

        path.push(".kube");
        path.push(DEFAULT_CONFIG_FILENAME);

        Ok(path)
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
}
