use crate::state::OptionalConfig;
use crate::types::minting_level::MintingLevel;
use crate::types::token_attributes::InputTokenAttributes;
use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use secret_toolkit::utils::types::Contract;
use serde::{Deserialize, Serialize};
use secret_toolkit::serialization::Base64JsonOf;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintPrice {
    pub token: Token,
    pub price: Uint128,
    pub whitelist_price: Uint128,
    pub items_price: Uint128,
    pub loot_box_price: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub nft_count: u16,
    pub nft_contract: Contract,
    pub random_seed: Binary,
    pub base_uri: String,
    pub price: Vec<MintPrice>,
    pub platform: Option<Contract>,
    pub only_platform: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistAddress {
    pub address: HumanAddr,
    pub amount: u8,
}



// #[derive(Deserialize, JsonSchema)]
// #[serde(rename_all = "snake_case")]

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    AddWhitelist {
        addresses: Vec<WhitelistAddress>,
    },
    RemoveWhitelist {
        addresses: Vec<HumanAddr>,
    },
    SetPlaceHolder {
        token_uri: String,
    },
    Mint {
        amount: Option<u8>, // avatar
        amount_loot_box_to_mint: Option<u8>,
        amount_item_to_mint: Option<u8>
    },
    MintAdmin {
        amount: Option<u8>, // avatar
        amount_loot_box_to_mint: Option<u8>,
        amount_item_to_mint: Option<u8>,
        mint_for: Option<HumanAddr>,
    },
    EnableReveal {},
    ChangingMintingState {
        mint_state: MintingLevel,
        cap_amount: Option<u16>,
    },
    OpenLootBox {
        token_id: String,
    },
    Receive {
        from: HumanAddr,
        msg: Option<Binary>,
        //msg: Base64JsonOf<ReceiveMsg>,
        amount: Uint128,
    },

    SetAttributes {
        tokens: Vec<InputTokenAttributes>,
    },
    WithdrawFunds {
        dest: HumanAddr,
        token: Token,
        snip20_msg: Option<Binary>,
        amount: Uint128,
    },
    ChangeConfig {
        settings: OptionalConfig,
    },
    Cleanup {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Token {
    Snip20(Contract),
    Native(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Mint {}


#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(Deserialize))]
pub enum HandleAnswer {
    OpenLootBox { status: ResponseStatus },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    // GetCount returns the current count as a json-encoded number
    Remaining {},
    MintingLevel {},
    IsWhitelisted { address: HumanAddr },
    // full price
    // whitelist price
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CapAmountResponse {
    pub cap_amount: u16,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RemainingResponse {
    pub remaining: u32,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintingLevelResponse {
    pub minting_level: String,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IsWhitelistedResponse {
    pub is_whitelisted: bool,
    pub amount: Option<u8>,
}

#[derive(Deserialize, JsonSchema, Clone, Debug)]
#[cfg_attr(test, derive(Serialize))]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Mint {
        mint_for: HumanAddr,
        amount_avatar_to_mint: u8,
        amount_loot_box_to_mint: u8,
        amount_item_to_mint: u8,
    },
}

// #[derive(Deserialize, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub enum ReceiveMsg {
//     ReceiveFromPlatform {
//         from: HumanAddr,
//         msg: Base64JsonOf<ReceiveFromPlatformMsg>,
//     },
// }

// #[derive(Deserialize, JsonSchema)]
// pub enum ReceiveFromPlatformMsg {
//     Mint {
//         mint_for: HumanAddr,
//         amount_avatar_to_mint: u8,
//         amount_loot_box_to_mint: u8,
//         amount_item_to_mint: u8,
//     },
// }

#[derive(Deserialize, JsonSchema)]
#[cfg_attr(test, derive(Serialize))]
#[serde(rename_all = "snake_case")]
pub enum PlatformApi {
    ReceiveFromPlatform { from: HumanAddr, msg: Binary },
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(Deserialize))]
pub enum ResponseStatus {
    Success,
    Failure,
}