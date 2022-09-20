use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::config::{Config, ContractMode};
use cosmwasm_std::{
    to_binary, Api, Binary, Extern, HumanAddr, Querier, StdError, StdResult, Storage, Uint128,
};
use secret_toolkit::permit::{validate, Permit};
use secret_toolkit::snip20;
use secret_toolkit::viewing_key::{ViewingKey, ViewingKeyStore};

use crate::vesting::{Schedule, Vesting};

#[allow(clippy::large_enum_variant)]
#[derive(Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Public(PublicQueryMsg),
    WithAuth(AuthQuery),
}

#[derive(Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AuthQuery {
    auth: Auth,
    query: AuthQueryMsg,
}

impl AuthQuery {
    fn check<S: Storage, A: Api, Q: Querier>(
        self,
        deps: &Extern<S, A, Q>,
        config: &Config,
    ) -> StdResult<(HumanAddr, AuthQueryMsg)> {
        let address = self.auth.check(deps, config)?;
        Ok((address, self.query))
    }
}

pub const PREFIX_REVOKED_PERMITS: &str = "revoked_permits";

#[derive(Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Auth {
    ViewingKey { address: HumanAddr, key: String },
    Permit(Permit),
}

impl Auth {
    /// Check if the auth parameters check out, and return the authorized account
    fn check<S: Storage, A: Api, Q: Querier>(
        self,
        deps: &Extern<S, A, Q>,
        config: &Config,
    ) -> StdResult<HumanAddr> {
        match self {
            Self::ViewingKey { address, key } => {
                ViewingKey::check(&deps.storage, &address, &key)?;
                Ok(address)
            }
            Self::Permit(permit) => validate(
                deps,
                PREFIX_REVOKED_PERMITS,
                &permit,
                config.contract_address.clone(),
                None,
            )
            .map(HumanAddr),
        }
    }
}

#[derive(Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PublicQueryMsg {
    ContractMode {},
}

#[derive(Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthQueryMsg {
    Balance { time: Option<u64> },
    Admin(AdminQueryMsg),
}

#[derive(Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AdminQueryMsg {
    /// Get the status of the fund. Do we have enough funds in the account to cover everyone's vesting schedules?
    FundStatus {},
    BalanceOf {
        address: HumanAddr,
        time: Option<u64>,
    },
}

#[derive(Serialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryResp {
    // Public
    ContractMode(ContractMode),

    // User
    Balance {
        address: HumanAddr,
        available: Uint128,
        schedule: Schedule,
    },

    // Admin
    FundStatus {
        /// How much in total is allocated right now
        allocated: Uint128,
        /// Our current SNIP-20 balance
        reserve: Uint128,
    },
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    let config = Config::load(&deps.storage)?;
    match msg {
        QueryMsg::Public(msg) => match msg {
            PublicQueryMsg::ContractMode {} => get_contract_mode(config),
        },
        QueryMsg::WithAuth(auth_query) => {
            let (address, query) = auth_query.check(deps, &config)?;

            match query {
                AuthQueryMsg::Balance { time } => balance(deps, config, address, time),

                AuthQueryMsg::Admin(msg) => {
                    config.assert_admin(&address)?;

                    match msg {
                        AdminQueryMsg::FundStatus {} => funds_status(deps, &config),
                        AdminQueryMsg::BalanceOf { address, time } => {
                            balance(deps, config, address, time)
                        }
                    }
                }
            }
        }
    }
}

fn balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    mut config: Config,
    address: HumanAddr,
    time: Option<u64>,
) -> StdResult<Binary> {
    let schedule = Vesting::get_schedule(&deps.storage, &address)?
        .ok_or_else(|| StdError::generic_err("No vesting schedule found"))?;

    // Override the known last block time.
    // `.available_at()` only cares about `config.last_block.time`
    if let Some(time) = time {
        config.last_block.time = time;
    }
    let available = Uint128(schedule.available_at(&config.last_block));

    to_binary(&QueryResp::Balance {
        address,
        available,
        schedule: schedule.into(),
    })
}

fn get_contract_mode(config: Config) -> StdResult<Binary> {
    to_binary(&QueryResp::ContractMode(config.mode))
}

fn funds_status<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    config: &Config,
) -> StdResult<Binary> {
    let allocated = Uint128(Vesting::get_total_allocation(&deps.storage)?);

    let reserve = snip20::balance_query(
        &deps.querier,
        config.contract_address.clone(),
        config.vesting_token_vk.clone(),
        256,
        config.vesting_token.hash.clone(),
        config.vesting_token.address.clone(),
    )?
    .amount;

    to_binary(&QueryResp::FundStatus { reserve, allocated })
}
