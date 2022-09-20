use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_binary, Api, Binary, Extern, HumanAddr, Querier, StdResult, Storage, Uint128,
};

mod airdrop_claims;
mod get_account_info;

use crate::queries::get_account_info::query_get_account_info;
use airdrop_claims::query_airdrop_claims;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    AirdropWasClaimed {
        address: String,
    },
    /// Function that reads airdrop numbers for each account. Can only be read by the quest contract,
    /// which sets a password that only it knows
    GetAccountInfo {
        password: String,
        address: HumanAddr,
    },
}

// We define a custom struct for each query response
#[derive(Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AirdropClaimResponse {
    pub claimed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccountInfoResponse {
    pub account: AccountInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccountInfo {
    pub address: HumanAddr,
    pub amount: Uint128,
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::AirdropWasClaimed { address } => to_binary(&query_airdrop_claims(deps, address)?),
        QueryMsg::GetAccountInfo { password, address } => {
            to_binary(&query_get_account_info(deps, password, address)?)
        }
    }
}
