use anyhow::anyhow;
use k8s_insider_core::wireguard::keys::{InvalidWgKey, WgKey};
use k8s_openapi::serde::{Deserialize, Serialize};

use crate::regex;

use super::network::NetworkIdentifier;

#[derive(Serialize, Deserialize, Clone, Default, Debug, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TunnelIdentifier {
    pub network: NetworkIdentifier,
    pub name: String,
}

impl TunnelIdentifier {
    pub fn from_network_identifier(network: NetworkIdentifier, name: String) -> Self {
        Self { network, name }
    }

    pub fn from_wgconf_header(header: &str) -> anyhow::Result<Option<Self>> {
        let regex = regex!(
            r"^# k8s-insider::\[\[(?x)
                (?P<context>[-0-9A-Za-z.]+)::
                (?P<namespace>[-0-9A-Za-z.]+)::
                (?P<network>[-0-9A-Za-z.]+)::
                (?P<tunnel>[-0-9A-Za-z.]+)\]\]\s*$"
        );

        let captures = regex.captures(header);

        if let Some(captures) = captures {
            let context = captures
                .name("context")
                .ok_or(anyhow!("Missing context in the header!"))?
                .as_str();
            let namespace = captures
                .name("namespace")
                .ok_or(anyhow!("Missing namespace in the header!"))?
                .as_str();
            let network = captures
                .name("network")
                .ok_or(anyhow!("Missing network name in the header!"))?
                .as_str();
            let tunnel = captures
                .name("tunnel")
                .ok_or(anyhow!("Missing tunnel name in the header!"))?
                .as_str();

            Ok(Some(Self {
                network: NetworkIdentifier {
                    name: network.to_owned(),
                    namespace: namespace.to_owned(),
                    context: context.to_owned(),
                },
                name: tunnel.to_owned(),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn generate_wgconf_header(&self) -> String {
        format!(
            "# k8s-insider::[[{}::{}::{}::{}]]",
            self.network.context, self.network.namespace, self.network.name, self.name
        )
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TunnelConfig {
    pub name: String,
    pub private_key: String,
}

impl TunnelConfig {
    pub fn new(name: String, private_key: WgKey) -> Self {
        Self {
            name,
            private_key: private_key.to_base64(),
        }
    }

    pub fn try_get_wgkey(&self) -> Result<WgKey, InvalidWgKey> {
        WgKey::from_base64(&self.private_key)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::network::NetworkIdentifier;

    use super::TunnelIdentifier;

    #[test]
    fn from_wgconf_header_properly_parses_header() {
        test_header_parse("# k8s-insider::[[minikube::kube-insider::default::default-8lkprtfr6247fjmmgv0k266v7njtvaqvc8nji6nmjc4tetht6he0]]", TunnelIdentifier {
            network: NetworkIdentifier {
                context: "minikube".to_owned(),
                namespace: "kube-insider".to_owned(),
                name: "default".to_owned(),
            },
            name: "default-8lkprtfr6247fjmmgv0k266v7njtvaqvc8nji6nmjc4tetht6he0".to_owned()
        });

        test_header_parse(
            "# k8s-insider::[[e2er2o::what.you-gonna.-.d00::custom::dsf1j2j22j2j2lj2j]]",
            TunnelIdentifier {
                network: NetworkIdentifier {
                    context: "e2er2o".to_owned(),
                    namespace: "what.you-gonna.-.d00".to_owned(),
                    name: "custom".to_owned(),
                },
                name: "dsf1j2j22j2j2lj2j".to_owned(),
            },
        );
    }

    fn test_header_parse(header: &str, expected: TunnelIdentifier) {
        let id = TunnelIdentifier::from_wgconf_header(header).unwrap().unwrap();

        assert_eq!(id, expected);
    }
}
