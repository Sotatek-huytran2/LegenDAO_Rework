use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage,
    Uint128,
};
use secret_toolkit::snip20;
use secret_toolkit::viewing_key::{ViewingKey, ViewingKeyStore};

use crate::config::{Config, ContractMode};
use crate::types::Contract;
use crate::vesting::{NewSchedule, Vesting};

#[derive(Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // User commands
    /// Claim available funds from the vesting account.
    ///
    /// If `amount` is not specified, all available funds are withdrawn.
    Claim { amount: Option<Uint128> },

    // Authentication
    /// Create a random viewing key using entropy
    CreateViewingKey { entropy: String },
    /// Set a custom viewing key
    SetViewingKey { key: String },

    // Admin commands
    /// Set the address and hash of the vesting token contract
    SetVestingToken { contract: Contract, key: String },
    /// Set the viewing key for the vesting token
    SetVestingTokenViewingKey { key: String },
    /// Add a vesting account with its own schedule
    AddAccounts {
        accounts: Vec<(HumanAddr, NewSchedule)>,
    },
    /// Remove an address from the set of vesting accounts
    RemoveAccounts { accounts: Vec<HumanAddr> },
    /// Redeem all funds to admin account. requires mode == "emergency"
    EmergencyRedeemAll {},
    /// Change the operational mode of the contract
    SetContractMode { mode: ContractMode },
    /// Change the contract's admin address
    ChangeAdmin { address: HumanAddr },
}

#[derive(Serialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleResp {
    Success,
    ViewingKey(String),
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    use HandleMsg::*;

    let mut config = Config::load(&deps.storage)?;
    config.last_block = env.block.clone();

    if let ContractMode::PausedClaims | ContractMode::Emergency = config.mode {
        if let Claim { .. } | AddAccounts { .. } | RemoveAccounts { .. } = &msg {
            return Err(StdError::generic_err(
                "This operation is not permitted because claims are paused",
            ));
        }
    }

    let result = match msg {
        Claim { amount } => claim(deps, env, &config, amount),
        CreateViewingKey { entropy } => create_viewing_key(deps, env, entropy),
        SetViewingKey { key } => set_viewing_key(deps, env, key),
        SetVestingToken { contract, key } => set_vesting_token(env, &mut config, contract, key),
        SetVestingTokenViewingKey { key } => set_vesting_token_viewing_key(env, &mut config, key),
        AddAccounts { accounts } => add_accounts(deps, env, &config, accounts),
        RemoveAccounts { accounts } => remove_accounts(deps, env, &config, accounts),
        EmergencyRedeemAll {} => emergency_redeem(deps, &config, env),
        SetContractMode { mode } => set_contract_mode(env, &mut config, mode),
        ChangeAdmin { address } => change_admin(env, &mut config, address),
    };

    config.save(&mut deps.storage)?;

    result
}

fn claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: &Config,
    amount: Option<Uint128>,
) -> StdResult<HandleResponse> {
    let claimed = Vesting::update(&mut deps.storage, |vesting| {
        let mut schedule = vesting
            .get_schedule(&env.message.sender)?
            .ok_or_else(|| StdError::generic_err("You are not eligible for vesting rewards"))?;

        let claimed = match amount {
            Some(amount) => schedule.claim(amount.u128(), &env.block)?,
            None => schedule.claim_all(&env.block),
        };

        vesting.set_schedule(env.message.sender.clone(), schedule);

        Ok(claimed)
    })?;

    let message = snip20::transfer_msg(
        env.message.sender,
        Uint128(claimed),
        Some("Vesting claim".to_string()),
        None,
        256,
        config.vesting_token.hash.clone(),
        config.vesting_token.address.clone(),
    )?;

    let res = HandleResponse {
        messages: vec![message],
        log: vec![],
        data: Some(to_binary(&HandleResp::Success)?),
    };

    Ok(res)
}

fn create_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    entropy: String,
) -> StdResult<HandleResponse> {
    let viewing_key = ViewingKey::create(
        &mut deps.storage,
        &env,
        &env.message.sender,
        entropy.as_bytes(),
    );

    let res = HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleResp::ViewingKey(viewing_key))?),
    };

    Ok(res)
}

fn set_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    key: String,
) -> StdResult<HandleResponse> {
    ViewingKey::set(&mut deps.storage, &env.message.sender, &key);

    let res = HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleResp::Success)?),
    };

    Ok(res)
}

fn set_vesting_token(
    env: Env,
    config: &mut Config,
    contract: Contract,
    key: String,
) -> StdResult<HandleResponse> {
    config.assert_admin(&env.message.sender)?;

    config.vesting_token = contract;
    config.vesting_token_vk = key.clone();

    let set_vk_msg = snip20::set_viewing_key_msg(
        key,
        None,
        256,
        config.vesting_token.hash.clone(),
        config.vesting_token.address.clone(),
    )?;

    let res = HandleResponse {
        messages: vec![set_vk_msg],
        log: vec![],
        data: Some(to_binary(&HandleResp::Success)?),
    };

    Ok(res)
}

fn set_vesting_token_viewing_key(
    env: Env,
    config: &mut Config,
    key: String,
) -> StdResult<HandleResponse> {
    config.assert_admin(&env.message.sender)?;

    config.vesting_token_vk = key.clone();

    let set_vk_msg = snip20::set_viewing_key_msg(
        key,
        None,
        256,
        config.vesting_token.hash.clone(),
        config.vesting_token.address.clone(),
    )?;

    let res = HandleResponse {
        messages: vec![set_vk_msg],
        log: vec![],
        data: Some(to_binary(&HandleResp::Success)?),
    };

    Ok(res)
}

fn add_accounts<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: &Config,
    accounts: Vec<(HumanAddr, NewSchedule)>,
) -> StdResult<HandleResponse> {
    config.assert_admin(&env.message.sender)?;

    let mut prev_schedules = vec![None; accounts.len()];
    Vesting::update(&mut deps.storage, |vesting| {
        for (index, (address, schedule)) in accounts.clone().into_iter().enumerate() {
            prev_schedules[index] = vesting.get_schedule(&address)?;

            vesting.set_schedule(address, schedule.into());
        }

        Ok(())
    })?;

    // If resetting an existing schedule, first claim the rest of the user's balance
    let mut messages = vec![];
    for (prev_schedule, (address, _)) in prev_schedules.into_iter().zip(accounts) {
        if let Some(mut prev_schedule) = prev_schedule {
            let to_claim = prev_schedule.claim_all(&env.block);
            messages.push(snip20::transfer_msg(
                address,
                Uint128(to_claim),
                Some("Vesting claim".to_string()),
                None,
                256,
                config.vesting_token.hash.clone(),
                config.vesting_token.address.clone(),
            )?);
        }
    }

    let res = HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleResp::Success)?),
    };

    Ok(res)
}

fn remove_accounts<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: &Config,
    accounts: Vec<HumanAddr>,
) -> StdResult<HandleResponse> {
    config.assert_admin(&env.message.sender)?;

    Vesting::update(&mut deps.storage, |vesting| {
        for address in accounts {
            vesting.remove_schedule(address);
        }
        Ok(())
    })?;

    let res = HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleResp::Success)?),
    };

    Ok(res)
}

fn emergency_redeem<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    config: &Config,
    env: Env,
) -> StdResult<HandleResponse> {
    config.assert_admin(&env.message.sender)?;

    if config.mode != ContractMode::Emergency {
        return Err(StdError::generic_err(
            "Contract mode must be set to Emergency in order to withdraw all funds",
        ));
    }

    let balance = snip20::balance_query(
        &deps.querier,
        env.contract.address,
        config.vesting_token_vk.clone(),
        256,
        config.vesting_token.hash.clone(),
        config.vesting_token.address.clone(),
    )?;

    let message = snip20::transfer_msg(
        config.admin.clone(),
        balance.amount,
        Some("Emergency redeem from $LGND vesting token".to_string()),
        None,
        256,
        config.vesting_token.hash.clone(),
        config.vesting_token.address.clone(),
    )?;

    let res = HandleResponse {
        messages: vec![message],
        log: vec![],
        data: Some(to_binary(&HandleResp::Success)?),
    };

    Ok(res)
}

fn set_contract_mode(
    env: Env,
    config: &mut Config,
    mode: ContractMode,
) -> StdResult<HandleResponse> {
    config.assert_admin(&env.message.sender)?;

    config.mode = mode;

    let res = HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleResp::Success)?),
    };

    Ok(res)
}

fn change_admin(env: Env, config: &mut Config, address: HumanAddr) -> StdResult<HandleResponse> {
    config.assert_admin(&env.message.sender)?;

    config.admin = address;

    let res = HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleResp::Success)?),
    };

    Ok(res)
}
