use crate::state::{Balances, Config, Features, RedeemInfo, TotalBalances, UnbondingRecord};
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
    pub token_native_denom: String,
    pub unbonding_period: Option<u64>,
    pub receiving_contracts: Option<Vec<HumanAddr>>,
    pub viewing_key: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    /// Withdraw funds from the platform, which will initiate an unbonding period
    Redeem {
        /// If not specified, use all funds
        amount: Option<Uint128>,
    },
    /// Manually claim funds that finished the unbonding time
    /// (i.e. don't wait for it to be claimed automatically)
    ClaimRedeemed {},
    /// Send tokens from platform to other contract in the LegenDAO ecosystem (e.g. NFT mint)
    SendFromPlatform {
        /// Destination contract
        contract_addr: HumanAddr,
        /// If not specified, use all funds
        amount: Option<Uint128>,
        /// Probably not necessary
        memo: Option<String>,
        /// Wanted message to initiate at the destination contract (defined in the destination contract)
        msg: Binary,
    },

    // Admin
    AddReceivingContracts {
        addresses: Vec<HumanAddr>,
    },
    RemoveReceivingContracts {
        addresses: Vec<HumanAddr>,
    },
    ChangeConfig {
        admin: Option<HumanAddr>,
        unbonding_period: Option<u64>,
    },

    // Viewing keys
    CreateViewingKey {
        entropy: String,
        padding: Option<String>,
    },
    SetViewingKey {
        key: String,
        padding: Option<String>,
    },

    // Permits
    RevokePermit {
        permit_name: String,
        padding: Option<String>,
    },

    // Snip20 commands
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        msg: Base64JsonOf<ReceiveMsg>,
    },

    // Feature toggle
    Features(FeatureToggleHandleMsg<Features>),
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    /// Deposit funds in platform
    Deposit {
        /// The account for which the funds will be deposited
        to: HumanAddr,
    },
    BatchDeposit(Vec<Deposit>),
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Deposit {
    pub to: HumanAddr,
    pub amount: Uint128,
}

impl Deposit {
    pub fn new(to: HumanAddr, amount: Uint128) -> Self {
        Self { to, amount }
    }
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(Deserialize))]
pub enum HandleAnswer {
    Deposit { status: ResponseStatus },
    Redeem { status: ResponseStatus },
    ClaimRedeemed { status: ResponseStatus },
    SendFromPlatform { status: ResponseStatus },
    AddReceivingContracts { status: ResponseStatus },
    RemoveReceivingContracts { status: ResponseStatus },
    CreateViewingKey { key: String },
    SetViewingKey { status: ResponseStatus },
    RevokePermit { status: ResponseStatus },
    SetPauser { status: ResponseStatus },
    RemovePauser { status: ResponseStatus },
    ChangeConfig { status: ResponseStatus },
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    /// Number of withdraws pending to be claimed (both unbonding and claimable)
    NumOfPendingClaims {},
    TotalBalances {},

    // Authenticated
    /// Balance of an account
    Balance {
        /// Address of the account
        address: HumanAddr,
        /// Viewing key of the account
        key: String,
    },

    // Permits
    /// Permit queries. See more: [Permits API](https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-24.md)
    WithPermit {
        permit: Permit,
        query: QueryWithPermit,
    },

    // Feature toggle
    Features(FeatureToggleQueryMsg<Features>),
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(Deserialize))]
pub enum QueryAnswer {
    Config(Config),
    Balance(ResponseBalances),
    NumOfPendingClaims(Uint128),
    TotalBalances(ResponseTotalBalances),
}

#[derive(Serialize, Deserialize, Default, Clone, JsonSchema)]
pub struct ResponseBalances {
    /// Staked amount, not including unbonding (or claimable) funds
    pub staked: Uint128,
    /// Withdraw requests
    pub pending_redeem: ResponseRedeemInfo,
}

impl From<Balances> for ResponseBalances {
    fn from(b: Balances) -> Self {
        Self {
            staked: b.staked.into(),
            pending_redeem: b.pending_redeem.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, JsonSchema)]
pub struct ResponseRedeemInfo {
    /// Unbonding withdraws
    unbondings: Vec<ResponseUnbondingRecord>, // Sorted by start time, bulked by day
    /// Claimable withdraws (i.e. finished unbonding period)
    claimable: Uint128,
}

impl From<RedeemInfo> for ResponseRedeemInfo {
    fn from(ri: RedeemInfo) -> Self {
        Self {
            unbondings: ri.unbondings.into_iter().map(|u| u.into()).collect(),
            claimable: ri.claimable.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, JsonSchema)]
pub struct ResponseUnbondingRecord {
    /// Unbonding period ending timestamp (in seconds)
    end_ts: u64,
    /// Amount unbonding
    amount: Uint128,
}

impl From<UnbondingRecord> for ResponseUnbondingRecord {
    fn from(ub: UnbondingRecord) -> Self {
        Self {
            end_ts: ub.end_ts,
            amount: ub.amount.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct ResponseTotalBalances {
    staked: Uint128,
    unbonding: Uint128,
}

impl From<TotalBalances> for ResponseTotalBalances {
    fn from(tb: TotalBalances) -> Self {
        Self {
            staked: tb.staked.into(),
            unbonding: tb.unbonding.into(),
        }
    }
}

impl QueryMsg {
    pub fn get_validation_params(&self) -> (&HumanAddr, String) {
        match self {
            QueryMsg::Balance { address, key } => (address, key.clone()),
            _ => panic!("This should never happen"),
        }
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryWithPermit {
    /// Balance of an account (the account that signed the permit). Same as QueryMsg::Balance
    Balance {},
}

#[derive(Serialize, JsonSchema)]
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
