use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{HumanAddr, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AirdropClaimSubmit {
    pub address: String,
    pub to: HumanAddr,
    pub amount: Uint128,
}
