use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::HumanAddr;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Contract {
    pub address: HumanAddr,
    pub hash: String,
}

impl Contract {
    #[allow(dead_code)]
    pub fn new(address: HumanAddr, hash: String) -> Self {
        Self { address, hash }
    }
}
