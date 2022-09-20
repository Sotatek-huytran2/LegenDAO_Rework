use std::cmp::{max, min};

use cosmwasm_std::{
    log, to_binary, Api, CosmosMsg, Env, Extern, HandleResponse, HandleResult, HumanAddr,
    InitResponse, InitResult, Querier, QueryResult, ReadonlyStorage, StdError, StdResult, Storage,
    Uint128, WasmMsg,
};
use primitive_types::U256;
use secret_toolkit::crypto::sha_256;
use secret_toolkit::permit::{validate, Permit, TokenPermissions};
use secret_toolkit::snip20;
use secret_toolkit::utils::feature_toggle::{
    FeatureStatus, FeatureToggle, FeatureToggleHandleMsg, FeatureToggleQueryMsg,
    FeatureToggleTrait, Status,
};
use secret_toolkit::utils::types::Contract;
use secret_toolkit::utils::{pad_handle_result, pad_query_result};
use secret_toolkit::viewing_key::{ViewingKey, ViewingKeyStore};

use crate::constants::*;
use crate::msg::ResponseStatus::{NotChanged, Success};
use crate::msg::{
    HandleAnswer, HandleMsg, InitMsg, QueryAnswer, QueryMsg, QueryWithPermit,
    ReceiveFromPlatformMsg, ReceiveMsg, SubscriberMsg,
};
use crate::state::{
    BoosterItem, BoosterItemInInventory, Config, Features, InflationSchedule, MultiplierContracts,
    RewardPool, ScheduleUnit, Subscribers, UserBalance, PREFIX_REVOKED_PERMITS,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> InitResult {
    // Initialize state
    let prng_seed_hashed = sha_256(&msg.prng_seed.0);

    let max_multiplier = msg.max_multiplier.unwrap_or(Uint128(300_000)).u128();
    if max_multiplier < MULTIPLIER_SCALE {
        return Err(StdError::generic_err(format!(
            "max_multiplier can't be less than {}",
            MULTIPLIER_SCALE
        )));
    }

    Config {
        admin: env.message.sender.clone(),
        token: msg.token.clone(),
        platform: msg.platform,
        viewing_key: msg.viewing_key.clone(),
        prng_seed: prng_seed_hashed.to_vec(),
        own_addr: env.contract.address,
        max_multiplier,
    }
    .save(&mut deps.storage)?;

    RewardPool {
        residue: 0,
        last_reward_block: env.block.height,
        total_locked: 0,
        total_weight: 0,
        acc_reward_per_share: 0,
    }
    .save(&mut deps.storage)?;
    InflationSchedule::save(
        &mut deps.storage,
        msg.inflation_schedule
            .iter()
            .map(|u| u.to_stored())
            .collect(),
    )?;
    Subscribers::save(&mut deps.storage, msg.subscribers.unwrap_or_default())?;

    if let Some(multiplier_contracts) = msg.multiplier_contracts {
        MultiplierContracts::add_multiple(&mut deps.storage, multiplier_contracts)?;
    }

    FeatureToggle::init_features(
        &mut deps.storage,
        vec![
            FeatureStatus {
                feature: Features::Deposit,
                status: Status::NotPaused,
            },
            FeatureStatus {
                feature: Features::Withdraw,
                status: Status::NotPaused,
            },
            FeatureStatus {
                feature: Features::EmergencyWithdraw,
                status: Status::Paused,
            },
            FeatureStatus {
                feature: Features::EmergencyWithdrawSkipPlatform,
                status: Status::Paused,
            },
        ],
        vec![env.message.sender],
    )?;

    Ok(InitResponse {
        messages: vec![
            snip20::register_receive_msg(
                env.contract_code_hash,
                None,
                1, // This is public data, no need to pad
                msg.token.hash.clone(),
                msg.token.address.clone(),
            )?,
            snip20::set_viewing_key_msg(
                msg.viewing_key,
                None,
                RESPONSE_BLOCK_SIZE,
                msg.token.hash,
                msg.token.address,
            )?,
        ],
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult {
    let response = match msg {
        // yo dawg
        HandleMsg::Receive {
            from, amount, msg, ..
        } => match msg.inner {
            ReceiveMsg::ReceiveFromPlatform { from: to, msg } => match msg.inner {
                ReceiveFromPlatformMsg::Deposit {} => deposit(deps, env, from, to, amount.u128()),
            },
        },
        HandleMsg::CreateViewingKey { entropy, .. } => create_viewing_key(deps, env, entropy),
        HandleMsg::SetViewingKey { key, .. } => set_viewing_key(deps, env, key),
        HandleMsg::ChangeAdmin { address } => change_admin(deps, env, address),
        HandleMsg::AddSubs { contracts } => add_subscribers(deps, env, contracts),
        HandleMsg::RemoveSubs { contracts } => remove_subscribers(deps, env, contracts),
        HandleMsg::AddMultiplierContracts { contracts } => {
            add_multiplier_contracts(deps, env, contracts)
        }
        HandleMsg::RemoveMultiplierContracts { contracts } => {
            remove_multiplier_contracts(deps, env, contracts)
        }
        HandleMsg::ApplyMultiplier {
            to,
            multiplier,
            item_id,
        } => apply_multiplier(deps, env, to, multiplier, item_id),
        HandleMsg::DropMultiplier { from, item_id } => drop_multiplier(deps, env, from, item_id),
        HandleMsg::Withdraw { amount } => withdraw(deps, env, amount.map(|some| some.u128())),
        HandleMsg::EmergencyWithdraw {} => emergency_withdraw(deps, env),
        HandleMsg::EmergencyWithdrawSkipPlatform {} => emergency_withdraw_skip_platform(deps, env),
        HandleMsg::Features(m) => match m {
            FeatureToggleHandleMsg::Pause { features } => {
                FeatureToggle::handle_pause(deps, &env, features)
            }
            FeatureToggleHandleMsg::Unpause { features } => {
                FeatureToggle::handle_unpause(deps, &env, features)
            }
            FeatureToggleHandleMsg::SetPauser { address } => set_pauser(deps, env, address),
            FeatureToggleHandleMsg::RemovePauser { address } => remove_pauser(deps, env, address),
        },
        HandleMsg::ChangeConfig {
            admin,
            platform,
            token_vk,
            inflation,
            max_multiplier,
        } => change_config(
            deps,
            env,
            admin,
            platform,
            token_vk,
            inflation,
            max_multiplier,
        ),
    };

    pad_handle_result(response, RESPONSE_BLOCK_SIZE)
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    let response = match msg {
        QueryMsg::Token {} => query_token_contract(deps),
        QueryMsg::TotalLocked {} => query_total_locked(deps),
        QueryMsg::Subscribers {} => query_subscribers(deps),
        QueryMsg::MultiplierContracts {
            page_number,
            page_size,
        } => query_multiplier_contracts(deps, page_number, page_size),
        QueryMsg::Admin {} => query_admin(deps),
        QueryMsg::Platform {} => query_platform_contract(deps),
        QueryMsg::InflationSchedule {} => query_inflation_schedule(deps),

        QueryMsg::WithPermit { permit, query } => permit_queries(deps, permit, query),
        QueryMsg::Features(m) => match m {
            FeatureToggleQueryMsg::Status { features } => {
                FeatureToggle::query_status(deps, features)
            }
            FeatureToggleQueryMsg::IsPauser { address } => {
                FeatureToggle::query_is_pauser(deps, address)
            }
        },
        QueryMsg::ContractBalanceFromSnip { key } => query_contract_balance_from_snip(deps, key),
        _ => authenticated_queries(deps, msg),
    };

    pad_query_result(response, RESPONSE_BLOCK_SIZE)
}

pub fn authenticated_queries<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> QueryResult {
    let (address, key) = msg.get_validation_params();
    ViewingKey::check(&deps.storage, address, &key)?;

    match msg {
        QueryMsg::Rewards {
            address, height, ..
        } => query_pending_rewards(deps, &address, height),
        QueryMsg::Balance { address, .. } => query_deposit(deps, &address),
        QueryMsg::BoosterItems {
            page_number,
            page_size,
            ..
        } => query_booster_items(deps, address, page_number, page_size),
        _ => Err(StdError::generic_err("unsupported authenticated query")),
    }
}

// Handle functions

fn deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    from: HumanAddr,
    to: HumanAddr,
    amount: u128,
) -> HandleResult {
    FeatureToggle::require_not_paused(&deps.storage, vec![Features::Deposit])?;

    // Ensure that the sent tokens are from an expected contract address
    let config = Config::load(&deps.storage)?;
    if env.message.sender != config.token.address {
        return Err(StdError::generic_err(format!(
            "this token is not supported. Supported: {}, given: {}",
            config.token.address, env.message.sender
        )));
    }
    // Ensure that the sender is the Platform contract
    if from != config.platform.address {
        return Err(StdError::generic_err(format!(
            "only the Platform contract ({}) is allowed to perform deposits",
            config.platform.address,
        )));
    }

    let mut messages: Vec<CosmosMsg> = vec![];
    let mut reward_pool = update_rewards(&deps.storage, env.block.height)?;
    let mut user_balance = UserBalance::load(&deps.storage, &to)?.unwrap_or_default();

    let real_multiplier = min(user_balance.total_multiplier, config.max_multiplier);
    let current_user_weight = U256::from(user_balance.weight);

    if user_balance.locked > 0 {
        let rewards = current_user_weight * U256::from(reward_pool.acc_reward_per_share)
            / U256::from(REWARD_SCALE)
            - U256::from(user_balance.debt);
        if rewards.as_u128() > 0 {
            messages.push(snip20::send_msg(
                config.platform.address,
                Uint128(rewards.as_u128()),
                Some(to_binary(&platform::msg::ReceiveMsg::Deposit {
                    to: to.clone(),
                })?),
                None,
                None,
                RESPONSE_BLOCK_SIZE,
                config.token.hash,
                config.token.address,
            )?);
        }
    }

    user_balance.locked += amount;
    let new_user_weight = U256::from(user_balance.locked) * U256::from(real_multiplier);
    let debt =
        new_user_weight * U256::from(reward_pool.acc_reward_per_share) / U256::from(REWARD_SCALE);

    user_balance.debt = debt.as_u128();
    user_balance.weight = new_user_weight.as_u128();
    user_balance.save(&mut deps.storage, &to)?;

    reward_pool.total_locked += amount;
    reward_pool.total_weight -= current_user_weight.as_u128();
    reward_pool.total_weight += new_user_weight.as_u128();
    reward_pool.save(&mut deps.storage)?;

    let subs = Subscribers::load(&deps.storage)?;
    let sub_messages: StdResult<Vec<CosmosMsg>> = subs
        .into_iter()
        .map(|s| create_subscriber_msg(s, &to, user_balance.locked))
        .collect();
    messages.extend(sub_messages?);

    Ok(HandleResponse {
        messages,
        log: vec![
            log("new_balance", user_balance.locked),
            log("new_weight", user_balance.weight),
        ],
        data: Some(to_binary(&HandleAnswer::Deposit { status: Success })?),
    })
}

fn withdraw<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Option<u128>,
) -> HandleResult {
    FeatureToggle::require_not_paused(&deps.storage, vec![Features::Withdraw])?;

    let config = Config::load(&deps.storage)?;
    let mut user_balance =
        UserBalance::load(&deps.storage, &env.message.sender)?.unwrap_or_default();
    let amount = amount.unwrap_or(user_balance.locked);

    if amount > user_balance.locked {
        return Err(StdError::generic_err(format!(
            "insufficient funds to redeem: balance={}, required={}",
            user_balance.locked, amount,
        )));
    }

    let mut reward_pool = update_rewards(&deps.storage, env.block.height)?;
    let real_multiplier = min(user_balance.total_multiplier, config.max_multiplier);
    let current_user_weight = U256::from(user_balance.weight);

    let rewards = current_user_weight * U256::from(reward_pool.acc_reward_per_share)
        / U256::from(REWARD_SCALE)
        - U256::from(user_balance.debt);

    user_balance.locked -= amount;
    let new_user_weight = U256::from(user_balance.locked) * U256::from(real_multiplier);
    let debt =
        new_user_weight * U256::from(reward_pool.acc_reward_per_share) / U256::from(REWARD_SCALE);
    user_balance.debt = debt.as_u128();
    user_balance.weight = new_user_weight.as_u128();
    user_balance.save(&mut deps.storage, &env.message.sender)?;

    reward_pool.total_locked -= amount;
    reward_pool.total_weight -= current_user_weight.as_u128();
    reward_pool.total_weight += new_user_weight.as_u128();
    reward_pool.save(&mut deps.storage)?;

    let mut messages = vec![snip20::send_msg(
        config.platform.address,
        Uint128(rewards.as_u128() + amount),
        Some(to_binary(&platform::msg::ReceiveMsg::Deposit {
            to: env.message.sender.clone(),
        })?),
        None,
        None,
        RESPONSE_BLOCK_SIZE,
        config.token.hash,
        config.token.address,
    )?];

    let subs = Subscribers::load(&deps.storage)?;
    let sub_messages: StdResult<Vec<CosmosMsg>> = subs
        .into_iter()
        .map(|s| create_subscriber_msg(s, &env.message.sender, user_balance.locked))
        .collect();
    messages.extend(sub_messages?);

    Ok(HandleResponse {
        messages,
        log: vec![
            log("new_balance", user_balance.locked),
            log("new_weight", user_balance.weight),
        ],
        data: Some(to_binary(&HandleAnswer::Redeem { status: Success })?),
    })
}

pub fn create_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    entropy: String,
) -> HandleResult {
    let key = ViewingKey::create(
        &mut deps.storage,
        &env,
        &env.message.sender,
        entropy.as_bytes(),
    );

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CreateViewingKey { key })?),
    })
}

pub fn set_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    key: String,
) -> HandleResult {
    ViewingKey::set(&mut deps.storage, &env.message.sender, &key);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetViewingKey { status: Success })?),
    })
}

fn change_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    address: HumanAddr,
) -> StdResult<HandleResponse> {
    let mut config = Config::load(&deps.storage)?;

    require_admin(&config, &env)?;

    config.admin = address;
    config.save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ChangeAdmin { status: Success })?),
    })
}

/// YOU SHOULD NEVER USE THIS! This will erase any eligibility for rewards you earned so far
fn emergency_withdraw<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> HandleResult {
    FeatureToggle::require_not_paused(&deps.storage, vec![Features::EmergencyWithdraw])?;

    let config = Config::load(&deps.storage)?;
    let user_balance = UserBalance::load(&deps.storage, &env.message.sender)?.unwrap_or_default();

    let mut reward_pool = RewardPool::load(&deps.storage)?;

    let mut messages = vec![];
    if user_balance.locked > 0 {
        messages.push(snip20::send_msg(
            config.platform.address,
            Uint128(user_balance.locked),
            Some(to_binary(&platform::msg::ReceiveMsg::Deposit {
                to: env.message.sender.clone(),
            })?),
            None,
            None,
            RESPONSE_BLOCK_SIZE,
            config.token.hash,
            config.token.address,
        )?);

        reward_pool.total_locked -= user_balance.locked;
        reward_pool.total_weight -= user_balance.weight;
        reward_pool.save(&mut deps.storage)?;
    }

    let new_user_balance = UserBalance {
        locked: 0,
        debt: 0,
        weight: 0,
        total_multiplier: user_balance.total_multiplier,
    };
    new_user_balance.save(&mut deps.storage, &env.message.sender)?;

    let subs = Subscribers::load(&deps.storage)?;
    let sub_messages: StdResult<Vec<CosmosMsg>> = subs
        .into_iter()
        .map(|s| create_subscriber_msg(s, &env.message.sender, user_balance.locked))
        .collect();
    messages.extend(sub_messages?);

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::EmergencyWithdraw {
            status: Success,
        })?),
    })
}

/// YOU SHOULD NEVER USE THIS! This will erase any eligibility for rewards you earned so far
fn emergency_withdraw_skip_platform<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> HandleResult {
    FeatureToggle::require_not_paused(
        &deps.storage,
        vec![Features::EmergencyWithdrawSkipPlatform],
    )?;

    let config = Config::load(&deps.storage)?;
    let mut user_balance =
        UserBalance::load(&deps.storage, &env.message.sender)?.unwrap_or_default();

    let mut reward_pool = RewardPool::load(&deps.storage)?;

    let mut messages = vec![];
    if user_balance.locked > 0 {
        messages.push(snip20::transfer_msg(
            env.message.sender.clone(),
            Uint128(user_balance.locked),
            None,
            None,
            RESPONSE_BLOCK_SIZE,
            config.token.hash,
            config.token.address,
        )?);

        reward_pool.total_locked -= user_balance.locked;
        reward_pool.total_weight -= user_balance.weight;
        reward_pool.save(&mut deps.storage)?;
    }

    user_balance = UserBalance {
        locked: 0,
        debt: 0,
        weight: 0,
        total_multiplier: user_balance.total_multiplier,
    };
    user_balance.save(&mut deps.storage, &env.message.sender)?;

    let subs = Subscribers::load(&deps.storage)?;
    let sub_messages: StdResult<Vec<CosmosMsg>> = subs
        .into_iter()
        .map(|s| create_subscriber_msg(s, &env.message.sender, user_balance.locked))
        .collect();
    messages.extend(sub_messages?);

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::EmergencyWithdrawSkipPlatform {
            status: Success,
        })?),
    })
}

fn add_subscribers<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    new_subs: Vec<Contract>,
) -> HandleResult {
    let config = Config::load(&deps.storage)?;
    require_admin(&config, &env)?;

    let mut subs = Subscribers::load(&deps.storage)?;
    subs.extend(new_subs);
    Subscribers::save(&mut deps.storage, subs)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddSubs { status: Success })?),
    })
}

fn remove_subscribers<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    subs_to_remove: Vec<HumanAddr>,
) -> StdResult<HandleResponse> {
    let config = Config::load(&deps.storage)?;
    require_admin(&config, &env)?;

    let mut subs = Subscribers::load(&deps.storage)?;
    subs = subs
        .into_iter()
        .filter(|s| !subs_to_remove.contains(&s.address))
        .collect();
    Subscribers::save(&mut deps.storage, subs)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RemoveSubs { status: Success })?),
    })
}

fn add_multiplier_contracts<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    contracts: Vec<HumanAddr>,
) -> HandleResult {
    let config = Config::load(&deps.storage)?;
    require_admin(&config, &env)?;

    MultiplierContracts::add_multiple(&mut deps.storage, contracts)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddMultiplierContracts {
            status: Success,
        })?),
    })
}

fn remove_multiplier_contracts<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    contracts: Vec<HumanAddr>,
) -> HandleResult {
    let config = Config::load(&deps.storage)?;
    require_admin(&config, &env)?;

    MultiplierContracts::remove_multiple(&mut deps.storage, contracts)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RemoveMultiplierContracts {
            status: Success,
        })?),
    })
}

fn apply_multiplier<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    to: HumanAddr,
    multiplier: u32,
    item_id: String,
) -> HandleResult {
    if multiplier < MULTIPLIER_SCALE as u32 {
        return Err(StdError::generic_err(
            "The multiplier cannot be smaller than 1",
        ));
    }
    MultiplierContracts::require_multiplier_contract(&deps.storage, &env.message.sender)?;
    if BoosterItem::is_locked(&deps.storage, &env.message.sender, &item_id)? {
        return Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            data: Some(to_binary(&HandleAnswer::ApplyMultiplier {
                status: NotChanged,
            })?),
        });
    }

    let new_item = BoosterItem {
        owner: to.clone(),
        multiplier,
    };
    new_item.save(&mut deps.storage, &env.message.sender, &item_id)?;

    let config = Config::load(&deps.storage)?;
    let mut messages: Vec<CosmosMsg> = vec![];
    let mut reward_pool = update_rewards(&deps.storage, env.block.height)?;
    let mut user_balance = UserBalance::load(&deps.storage, &to)?.unwrap_or_default();

    let current_user_weight = U256::from(user_balance.weight);

    if user_balance.locked > 0 {
        let rewards = current_user_weight * U256::from(reward_pool.acc_reward_per_share)
            / U256::from(REWARD_SCALE)
            - U256::from(user_balance.debt);
        if rewards.as_u128() > 0 {
            messages.push(snip20::send_msg(
                config.platform.address,
                Uint128(rewards.as_u128()),
                Some(to_binary(&platform::msg::ReceiveMsg::Deposit {
                    to: to.clone(),
                })?),
                None,
                None,
                RESPONSE_BLOCK_SIZE,
                config.token.hash,
                config.token.address,
            )?);
        }
    }

    let total_multiplier = U256::from(user_balance.total_multiplier) - U256::from(MULTIPLIER_SCALE)
        + U256::from(multiplier as u128);
    user_balance.total_multiplier = total_multiplier.as_u128();
    let real_multiplier = min(user_balance.total_multiplier, config.max_multiplier);

    let new_user_weight = U256::from(user_balance.locked) * U256::from(real_multiplier);
    let debt =
        new_user_weight * U256::from(reward_pool.acc_reward_per_share) / U256::from(REWARD_SCALE);
    user_balance.debt = debt.as_u128();
    user_balance.weight = new_user_weight.as_u128();
    user_balance.save(&mut deps.storage, &to)?;

    reward_pool.total_weight -= current_user_weight.as_u128();
    reward_pool.total_weight += new_user_weight.as_u128();
    reward_pool.save(&mut deps.storage)?;

    let subs = Subscribers::load(&deps.storage)?;
    let sub_messages: StdResult<Vec<CosmosMsg>> = subs
        .into_iter()
        .map(|s| create_subscriber_msg(s, &to, user_balance.locked))
        .collect();
    messages.extend(sub_messages?);

    Ok(HandleResponse {
        messages,
        log: vec![
            log("new_balance", user_balance.locked),
            log("new_weight", user_balance.weight),
        ],
        data: Some(to_binary(&HandleAnswer::ApplyMultiplier {
            status: Success,
        })?),
    })
}

fn drop_multiplier<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    from: HumanAddr,
    item_id: String,
) -> HandleResult {
    MultiplierContracts::require_multiplier_contract(&deps.storage, &env.message.sender)?;
    let may_multiplier =
        BoosterItem::get_multiplier(&deps.storage, &env.message.sender, &item_id, &from)?;

    let mut messages: Vec<CosmosMsg> = vec![];
    let mut user_balance = UserBalance::load(&deps.storage, &from)?.unwrap_or_default();

    let response = match may_multiplier {
        None => {
            // if the item was already dropped, do nothing instead of raising an error, in case this
            // contract was added as a subscriber of the nft contract when the item was already locked.
            HandleResponse {
                messages: vec![],
                log: vec![],
                data: Some(to_binary(&HandleAnswer::DropMultiplier {
                    status: NotChanged,
                })?),
            }
        }
        Some(multiplier) => {
            BoosterItem::remove(&mut deps.storage, &env.message.sender, &item_id, &from)?;

            let config = Config::load(&deps.storage)?;
            let mut reward_pool = update_rewards(&deps.storage, env.block.height)?;

            let current_user_weight = U256::from(user_balance.weight);

            if user_balance.locked > 0 {
                let rewards = current_user_weight * U256::from(reward_pool.acc_reward_per_share)
                    / U256::from(REWARD_SCALE)
                    - U256::from(user_balance.debt);
                if rewards.as_u128() > 0 {
                    messages.push(snip20::send_msg(
                        config.platform.address,
                        Uint128(rewards.as_u128()),
                        Some(to_binary(&platform::msg::ReceiveMsg::Deposit {
                            to: from.clone(),
                        })?),
                        None,
                        None,
                        RESPONSE_BLOCK_SIZE,
                        config.token.hash,
                        config.token.address,
                    )?);
                }
            }

            let total_multiplier = U256::from(user_balance.total_multiplier)
                - U256::from(multiplier as u128)
                + U256::from(MULTIPLIER_SCALE);

            user_balance.total_multiplier = max(total_multiplier.as_u128(), MULTIPLIER_SCALE); // can`t go below 1

            let real_multiplier = min(user_balance.total_multiplier, config.max_multiplier);
            let new_user_weight = U256::from(user_balance.locked) * U256::from(real_multiplier);
            let debt = new_user_weight * U256::from(reward_pool.acc_reward_per_share)
                / U256::from(REWARD_SCALE);
            user_balance.debt = debt.as_u128();
            user_balance.weight = new_user_weight.as_u128();
            user_balance.save(&mut deps.storage, &from)?;

            reward_pool.total_weight -= current_user_weight.as_u128();
            reward_pool.total_weight += new_user_weight.as_u128();
            reward_pool.save(&mut deps.storage)?;

            let subs = Subscribers::load(&deps.storage)?;
            let sub_messages: StdResult<Vec<CosmosMsg>> = subs
                .into_iter()
                .map(|s| create_subscriber_msg(s, &from, user_balance.locked))
                .collect();
            messages.extend(sub_messages?);

            HandleResponse {
                messages,
                log: vec![
                    log("new_balance", user_balance.locked),
                    log("new_weight", user_balance.weight),
                ],
                data: Some(to_binary(&HandleAnswer::DropMultiplier {
                    status: Success,
                })?),
            }
        }
    };

    Ok(response)
}

fn set_pauser<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    address: HumanAddr,
) -> HandleResult {
    let config = Config::load(&deps.storage)?;
    require_admin(&config, &env)?;

    FeatureToggle::handle_set_pauser(deps, &env, address)
}

fn remove_pauser<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    address: HumanAddr,
) -> HandleResult {
    let config = Config::load(&deps.storage)?;
    require_admin(&config, &env)?;

    FeatureToggle::handle_remove_pauser(deps, &env, address)
}

fn change_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    admin: Option<HumanAddr>,
    platform: Option<Contract>,
    token_vk: Option<String>,
    inflation: Option<Vec<ScheduleUnit>>,
    max_multiplier: Option<Uint128>,
) -> HandleResult {
    let mut config = Config::load(&deps.storage)?;
    require_admin(&config, &env)?;

    let mut messages = vec![];

    if let Some(admin) = admin {
        config.admin = admin;
    }

    if let Some(platform) = platform {
        config.platform = platform;
    }

    if let Some(token_vk) = token_vk {
        messages.push(snip20::set_viewing_key_msg(
            token_vk.clone(),
            None,
            RESPONSE_BLOCK_SIZE,
            config.token.hash.clone(),
            config.token.address.clone(),
        )?);
        config.viewing_key = token_vk;
    }

    if let Some(inflation) = inflation {
        let reward_pool = update_rewards(&deps.storage, env.block.height)?;
        reward_pool.save(&mut deps.storage)?;
        InflationSchedule::save(
            &mut deps.storage,
            inflation.iter().map(|u| u.to_stored()).collect(),
        )?;
    }

    if let Some(max_multiplier) = max_multiplier {
        if max_multiplier.u128() < MULTIPLIER_SCALE {
            return Err(StdError::generic_err(
                "max multiplier cannot be smaller than 1",
            ));
        }

        // Note that even though the configuration's max_multiplier changes, the multiplier that
        // applies to users will not instantly become limited by this value. This is because, in
        // order to avoid iteration, we apply the change to each user's weight individually, and
        // ONLY WHEN they cause a change to the locked/multiplier amount (on deposit / withdraw /
        // apply / drop_multiplier).
        config.max_multiplier = max_multiplier.u128();
    }

    config.save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ChangeConfig { status: Success })?),
    })
}

// Query functions

fn permit_queries<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    permit: Permit,
    query: QueryWithPermit,
) -> QueryResult {
    // Validate permit content
    let self_addr = Config::load(&deps.storage)?.own_addr;

    let account = &HumanAddr(validate(
        deps,
        PREFIX_REVOKED_PERMITS,
        &permit,
        self_addr,
        None,
    )?);

    // Permit validated! We can now execute the query.
    match query {
        QueryWithPermit::Balance {} => {
            if !permit.check_permission(&TokenPermissions::Balance) {
                return Err(StdError::generic_err(format!(
                    "No permission to query balance, got permissions {:?}",
                    permit.params.permissions
                )));
            }

            query_deposit(deps, account)
        }
        QueryWithPermit::Rewards { height } => {
            if !permit.check_permission(&TokenPermissions::Allowance) {
                return Err(StdError::generic_err(format!(
                    "No permission to query rewards (allowance permission), got permissions {:?}",
                    permit.params.permissions
                )));
            }

            query_pending_rewards(deps, account, height)
        }
        QueryWithPermit::ItemsLocked {
            page_number,
            page_size,
        } => {
            if !permit.check_permission(&TokenPermissions::Allowance) {
                return Err(StdError::generic_err(format!(
                    "No permission to query items locked (history permission), got permissions {:?}",
                    permit.params.permissions
                )));
            }

            query_booster_items(deps, account, page_number, page_size)
        }
    }
}

fn query_booster_items<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
    page_number: Option<u32>,
    page_size: u32,
) -> QueryResult {
    let start_page = page_number.unwrap_or(0);
    let page =
        BoosterItemInInventory::get_inventory_page(&deps.storage, account, start_page, page_size)?;

    to_binary(&QueryAnswer::BoosterItems { items: page })
}

fn query_pending_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr,
    block: u64,
) -> QueryResult {
    let new_reward_pool = update_rewards(&deps.storage, block)?;
    let user_balance = UserBalance::load(&deps.storage, address)?.unwrap_or_default();

    to_binary(&QueryAnswer::Rewards {
        rewards: Uint128(
            ((U256::from(user_balance.weight) * U256::from(new_reward_pool.acc_reward_per_share)
                / U256::from(REWARD_SCALE))
                - U256::from(user_balance.debt))
            .as_u128(),
        ),
    })
}

fn query_deposit<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr,
) -> QueryResult {
    let user_balance = UserBalance::load(&deps.storage, address)?.unwrap_or_default();

    // effective multiplier does not always reflect "min(config.max_multiplier, user_balance.total_multiplier)",
    // because configuration may have changed globally but not yet applied to users. It only applies to users
    // once they make a change in the locked/multiplier amount.
    let mut effective_multiplier = MULTIPLIER_SCALE;
    if user_balance.locked > 0 {
        effective_multiplier = user_balance.weight / user_balance.locked;
    }

    to_binary(&QueryAnswer::Balance {
        amount: Uint128(user_balance.locked),
        total_multiplier: Uint128(user_balance.total_multiplier),
        effective_multiplier: Uint128(effective_multiplier),
    })
}

fn query_token_contract<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let config = Config::load(&deps.storage)?;

    to_binary(&QueryAnswer::Token {
        contract: config.token,
    })
}

fn query_platform_contract<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let config = Config::load(&deps.storage)?;

    to_binary(&QueryAnswer::Platform {
        contract: config.platform,
    })
}

fn query_inflation_schedule<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let stored_inflation_schedule = InflationSchedule::load(&deps.storage)?;
    let inflation_schedule = stored_inflation_schedule
        .iter()
        .map(|u| u.from_stored())
        .collect();

    to_binary(&QueryAnswer::InflationSchedule { inflation_schedule })
}

fn query_total_locked<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let reward_pool = RewardPool::load(&deps.storage)?;

    to_binary(&QueryAnswer::TotalLocked {
        amount: Uint128(leave_n_most_significant_digits(reward_pool.total_locked, 3)),
        total_weight: Uint128(leave_n_most_significant_digits(reward_pool.total_weight, 3)),
    })
}

fn query_subscribers<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let subs = Subscribers::load(&deps.storage)?;

    to_binary(&QueryAnswer::Subscribers { contracts: subs })
}

fn query_multiplier_contracts<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    page_number: Option<u32>,
    page_size: u32,
) -> QueryResult {
    let multiplier_contracts =
        MultiplierContracts::get_page(&deps.storage, page_number, page_size)?;

    to_binary(&QueryAnswer::MultiplierContracts {
        contracts: multiplier_contracts,
    })
}

fn query_admin<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let config = Config::load(&deps.storage)?;

    to_binary(&QueryAnswer::Admin {
        address: config.admin,
    })
}

fn query_contract_balance_from_snip<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    key: String,
) -> QueryResult {
    let config = Config::load(&deps.storage)?;
    let balance = snip20::balance_query(
        &deps.querier,
        config.own_addr,
        key,
        RESPONSE_BLOCK_SIZE,
        config.token.hash,
        config.token.address,
    )?;

    to_binary(&QueryAnswer::ContractBalanceFromSnip {
        amount: balance.amount,
    })
}

// Helper functions

fn require_admin(config: &Config, env: &Env) -> StdResult<()> {
    if config.admin != env.message.sender {
        return Err(StdError::generic_err(format!(
            "not an admin: {}",
            env.message.sender
        )));
    }

    Ok(())
}

fn update_rewards<S: ReadonlyStorage>(storage: &S, block: u64) -> StdResult<RewardPool> {
    let mut reward_pool = RewardPool::load(storage)?;
    let new_rewards =
        InflationSchedule::get_inflation(storage, reward_pool.last_reward_block, block)?;

    if new_rewards == 0 {
        return Ok(reward_pool);
    }

    if reward_pool.total_weight == 0 {
        reward_pool.last_reward_block = block;
        reward_pool.residue += new_rewards;
        return Ok(reward_pool);
    }

    // Effectively distributes the residue to the first one that stakes to an empty pool
    reward_pool.acc_reward_per_share +=
        (new_rewards + reward_pool.residue) * REWARD_SCALE / reward_pool.total_weight;
    reward_pool.residue = 0;
    reward_pool.last_reward_block = block;

    Ok(reward_pool)
}

fn leave_n_most_significant_digits(num: u128, digits: u32) -> u128 {
    let base = 10_u128;

    // Check if `num` is big enough to meaningfully obfuscate
    if num == 0 || base.pow(digits) >= num {
        return num;
    }

    let mut min_oom = 0;
    let mut check_oom;
    let mut max_oom = 38;

    // Find `num`'s decimal order of magnitude
    while max_oom - min_oom > 1 {
        check_oom = min_oom + (max_oom - min_oom) / 2;

        if num / base.pow(check_oom) == 0 {
            max_oom = check_oom;
        } else {
            min_oom = check_oom;
        }
    }

    // if the min and max are 1 number apart, we need to check which is the right one.
    check_oom = match num / base.pow(min_oom) > base {
        true => max_oom,
        false => min_oom,
    };

    let filter = base.pow(1 + check_oom - digits);

    num / filter * filter
}

fn create_subscriber_msg(sub: Contract, user: &HumanAddr, new_stake: u128) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: sub.address,
        callback_code_hash: sub.hash,
        msg: to_binary(&SubscriberMsg::StakeChange {
            voter: user.clone(),
            new_stake: Uint128(new_stake),
        })?,
        send: vec![],
    }))
}
