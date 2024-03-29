use std::collections::BTreeMap;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use super::tunnel::TunnelConfig;

#[derive(Serialize, Deserialize, Clone, Default, Debug, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NetworkIdentifier {
    pub name: String,
    pub namespace: String,
    pub context: String,
}

impl NetworkIdentifier {
    pub fn new(name: String, namespace: String, context: String) -> Self {
        Self {
            name,
            namespace,
            context,
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NetworkConfig {
    #[serde(flatten)]
    pub id: NetworkIdentifier,
    tunnels: BTreeMap<String, TunnelConfig>,
}

impl NetworkConfig {
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

    pub fn list_tunnels(&self) -> impl Iterator<Item = (&String, &TunnelConfig)> {
        self.tunnels.iter()
    }

    pub fn try_add_tunnel(&mut self, name: String, config: TunnelConfig) -> anyhow::Result<()> {
        if self.tunnels.contains_key(&name) {
            return Err(anyhow!(
                "Tunnel named {name} is already present in the config!"
            ));
        }

        if self.tunnels.insert(name, config).is_some() {
            panic!("OVERTAKING IN A TUNNEL IS A DANGEROUS MANEUVER!");
        }

        Ok(())
    }

    pub fn try_remove_tunnel(&mut self, name: &str) -> anyhow::Result<TunnelConfig> {
        if !self.tunnels.contains_key(name) {
            return Err(anyhow!("There's no '{name}' tunnel in the config!"));
        }

        Ok(self.tunnels.remove(name).unwrap())
    }

    pub fn generate_config_tunnel_name(&self) -> String {
        for index in 0.. {
            let name = format!("tun{index}");

            if !self.tunnels.contains_key(&name) {
                return name;
            }
        }

        panic!("You disobeyed my orders son, why were you ever born?");
    }
}

impl From<NetworkIdentifier> for NetworkConfig {
    fn from(value: NetworkIdentifier) -> Self {
        Self {
            id: value,
            tunnels: BTreeMap::new(),
        }
    }
}

impl From<&NetworkIdentifier> for NetworkConfig {
    fn from(value: &NetworkIdentifier) -> Self {
        Self {
            id: value.to_owned(),
            tunnels: BTreeMap::new(),
        }
    }
}
