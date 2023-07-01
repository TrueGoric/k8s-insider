use std::collections::HashMap;

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ip::{addrpair::IpAddrPair, netpair::IpNetPair};

#[derive(CustomResource, Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[kube(
    group = "k8s-insider.dev",
    version = "v1alpha1",
    kind = "AddressAllocation",
    namespaced,
    derive = "Default"
)]
pub struct AddressAllocationSpec {
    pub subnet: IpNetPair,
    /// peer public key
    pub allocated_addresses: HashMap<String, IpAddrPair>,
}
