use std::convert::TryInto;

use schemars::JsonSchema;
use serde::Deserialize;

use cosmwasm_std::{
    Api, Binary, Context, Env, Extern, HumanAddr, InitResponse, Querier, StdResult, Storage,
};
use secret_toolkit::snip20;
use secret_toolkit::viewing_key::{ViewingKey, ViewingKeyStore};

use crate::config::Config;
use crate::types::Contract;
use crate::vesting::{NewSchedule, Vesting};

#[derive(Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    /// The administrator account, contract initializer by default
    admin: Option<HumanAddr>,
    /// The SNIP-20 token to distribute tokens from
    vesting_token: Contract,
    /// The vesting token's viewing key
    vesting_token_vk: String,
    /// List of vesting schedules
    schedules: Vec<(HumanAddr, NewSchedule)>,
    /// The initial seed for the Viewing Keys store
    prng_seed: Binary,
}

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let config = Config::new(
        msg.admin.unwrap_or(env.message.sender),
        env.contract.address,
        msg.vesting_token.clone(),
        msg.vesting_token_vk.clone(),
        env.block,
    );
    config.save(&mut deps.storage)?;

    let schedules = msg.schedules;
    Vesting::update(&mut deps.storage, |vesting| {
        for (address, schedule) in schedules {
            vesting.set_schedule(address, schedule.into());
        }
        Ok(())
    })?;

    ViewingKey::set_seed(&mut deps.storage, msg.prng_seed.as_slice());

    let mut response = Context::new();
    response.add_message(snip20::set_viewing_key_msg(
        msg.vesting_token_vk,
        None,
        256,
        msg.vesting_token.hash.clone(),
        msg.vesting_token.address.clone(),
    )?);
    // This one might be redundant
    response.add_message(snip20::register_receive_msg(
        env.contract_code_hash,
        None,
        256,
        msg.vesting_token.hash,
        msg.vesting_token.address,
    )?);
    response.try_into()
}
