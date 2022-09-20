use crate::state::{BoosterItemInInventory, Features, ScheduleUnit};
use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use secret_toolkit::permit::Permit;
use secret_toolkit::serialization::Base64JsonOf;
use secret_toolkit::utils::feature_toggle::{FeatureToggleHandleMsg, FeatureToggleQueryMsg};
use secret_toolkit::utils::types::Contract;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, JsonSchema)]
pub struct InitMsg {
    pub token: Contract,
    pub platform: Contract,
    pub inflation_schedule: Vec<ScheduleUnit>,
    pub viewing_key: String,
    pub prng_seed: Binary,
    pub subscribers: Option<Vec<Contract>>,
    pub max_multiplier: Option<Uint128>,
    pub multiplier_contracts: Option<Vec<HumanAddr>>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Withdraw {
        amount: Option<Uint128>,
    },
    CreateViewingKey {
        entropy: String,
        padding: Option<String>,
    },
    SetViewingKey {
        key: String,
        padding: Option<String>,
    },
    EmergencyWithdraw {},
    EmergencyWithdrawSkipPlatform {},

    // Registered commands
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        msg: Base64JsonOf<ReceiveMsg>,
    },

    Features(FeatureToggleHandleMsg<Features>),

    ApplyMultiplier {
        to: HumanAddr,
        multiplier: u32,
        item_id: String,
    },
    DropMultiplier {
        from: HumanAddr,
        item_id: String,
    },

    // Admin commands
    ChangeAdmin {
        address: HumanAddr,
    },
    AddSubs {
        contracts: Vec<Contract>,
    },
    RemoveSubs {
        contracts: Vec<HumanAddr>,
    },
    AddMultiplierContracts {
        contracts: Vec<HumanAddr>,
    },
    RemoveMultiplierContracts {
        contracts: Vec<HumanAddr>,
    },
    ChangeConfig {
        admin: Option<HumanAddr>,
        platform: Option<Contract>,
        token_vk: Option<String>,
        inflation: Option<Vec<ScheduleUnit>>,
        max_multiplier: Option<Uint128>,
    },
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
pub enum HandleAnswer {
    Redeem { status: ResponseStatus },
    CreateViewingKey { key: String },
    SetViewingKey { status: ResponseStatus },
    ChangeAdmin { status: ResponseStatus },
    EmergencyWithdraw { status: ResponseStatus },
    EmergencyWithdrawSkipPlatform { status: ResponseStatus },
    AddSubs { status: ResponseStatus },
    RemoveSubs { status: ResponseStatus },
    AddMultiplierContracts { status: ResponseStatus },
    RemoveMultiplierContracts { status: ResponseStatus },
    Deposit { status: ResponseStatus },
    ApplyMultiplier { status: ResponseStatus },
    DropMultiplier { status: ResponseStatus },
    ChangeConfig { status: ResponseStatus },
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    ReceiveFromPlatform {
        from: HumanAddr,
        msg: Base64JsonOf<ReceiveFromPlatformMsg>,
    },
}

#[derive(Deserialize, JsonSchema)]
pub enum ReceiveFromPlatformMsg {
    Deposit {},
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Admin {},
    TotalLocked {},
    Subscribers {},
    MultiplierContracts {
        page_number: Option<u32>,
        page_size: u32,
    },
    Features(FeatureToggleQueryMsg<Features>),
    ContractBalanceFromSnip {
        key: String,
    },
    Token {},
    Platform {},
    InflationSchedule {},

    // Authenticated
    Rewards {
        address: HumanAddr,
        key: String,
        height: u64,
    },
    Balance {
        address: HumanAddr,
        key: String,
    },
    BoosterItems {
        address: HumanAddr,
        key: String,
        page_number: Option<u32>,
        page_size: u32,
    },

    // Permits
    /// Permit queries. See more: [Permits API](https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-24.md)
    WithPermit {
        permit: Permit,
        query: QueryWithPermit,
    },
}

impl QueryMsg {
    pub fn get_validation_params(&self) -> (&HumanAddr, String) {
        match self {
            QueryMsg::Rewards { address, key, .. } => (address, key.clone()),
            QueryMsg::Balance { address, key } => (address, key.clone()),
            QueryMsg::BoosterItems { address, key, .. } => (address, key.clone()),
            _ => panic!("This should never happen"),
        }
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryWithPermit {
    /// Balance of an account (the account that signed the permit). Same as QueryMsg::Balance
    Balance {},
    Rewards {
        height: u64,
    },
    ItemsLocked {
        page_number: Option<u32>,
        page_size: u32,
    },
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum QueryAnswer {
    Admin {
        address: HumanAddr,
    },
    Rewards {
        rewards: Uint128,
    },
    Balance {
        amount: Uint128,
        total_multiplier: Uint128,
        effective_multiplier: Uint128,
    },
    BoosterItems {
        items: Vec<BoosterItemInInventory>,
    },
    TotalLocked {
        amount: Uint128,
        total_weight: Uint128,
    },
    Subscribers {
        contracts: Vec<Contract>,
    },
    MultiplierContracts {
        contracts: Vec<HumanAddr>,
    },
    ContractBalanceFromSnip {
        amount: Uint128,
    },
    Token {
        contract: Contract,
    },
    Platform {
        contract: Contract,
    },
    InflationSchedule {
        inflation_schedule: Vec<ScheduleUnit>,
    },

    QueryError {
        msg: String,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Failure,
    NotChanged,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriberMsg {
    StakeChange {
        voter: HumanAddr,
        new_stake: Uint128,
    },
}
