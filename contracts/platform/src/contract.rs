use crate::auto_claim::AutoClaims;
use crate::constants::{PREFIX_REVOKED_PERMITS, RESPONSE_BLOCK_SIZE};
use crate::msg::ResponseStatus::Success;
use crate::msg::{
    Deposit, HandleAnswer, HandleMsg, InitMsg, PlatformApi, QueryAnswer, QueryMsg, QueryWithPermit,
    ReceiveMsg, ResponseStatus,
};
use crate::state::{
    BalanceChange, Balances, Config, Features, ReceivingContracts, TotalBalances, SECONDS_IN_DAY,
};
use cosmwasm_std::{
    to_binary, Api, BankMsg, Binary, Coin, CosmosMsg, Env, Extern, HandleResponse, HandleResult,
    HumanAddr, InitResponse, InitResult, Querier, QueryResult, StdError, StdResult, Storage,
    Uint128,
};
use secret_toolkit::permit::{validate, Permit, RevokedPermits, TokenPermissions};
use secret_toolkit::snip20;
use secret_toolkit::utils::feature_toggle::{
    FeatureStatus, FeatureToggle, FeatureToggleHandleMsg, FeatureToggleQueryMsg, FeatureToggleTrait,
};
use secret_toolkit::viewing_key::{ViewingKey, ViewingKeyStore};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> InitResult {
    Config {
        admin: env.message.sender.clone(),
        token: msg.token.clone(),
        native_token_denom: msg.token_native_denom,
        unbonding_period: msg.unbonding_period.unwrap_or(SECONDS_IN_DAY * 21),
        self_contract_addr: env.contract.address,
    }
    .save(&mut deps.storage)?;

    if let Some(receiving_contracts) = msg.receiving_contracts {
        ReceivingContracts::set_multiple(&mut deps.storage, receiving_contracts)?;
    }

    FeatureToggle::init_features(
        &mut deps.storage,
        vec![
            FeatureStatus {
                feature: Features::Redeem,
                status: Default::default(),
            },
            FeatureStatus {
                feature: Features::Claim,
                status: Default::default(),
            },
            FeatureStatus {
                feature: Features::SendFromPlatform,
                status: Default::default(),
            },
            FeatureStatus {
                feature: Features::Deposit,
                status: Default::default(),
            },
        ],
        vec![env.message.sender],
    )?;

    Ok(InitResponse {
        messages: vec![
            snip20::register_receive_msg(
                env.contract_code_hash,
                None,
                1,
                msg.token.hash.clone(),
                msg.token.address.clone(),
            )?,
            snip20::set_viewing_key_msg(
                msg.viewing_key,
                None,
                1,
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
    let mut result = match msg {
        HandleMsg::Redeem { amount } => redeem(deps, &env, amount.map(|a| a.u128())),
        HandleMsg::ClaimRedeemed {} => claim(deps, &env),
        HandleMsg::SendFromPlatform {
            contract_addr,
            amount,
            memo,
            msg,
        } => send_from_platform(
            deps,
            &env,
            contract_addr,
            amount.map(|a| a.u128()),
            memo,
            msg,
        ),
        HandleMsg::AddReceivingContracts { addresses } => {
            add_receiving_contracts(deps, &env, addresses)
        }
        HandleMsg::RemoveReceivingContracts { addresses } => {
            remove_receiving_contracts(deps, &env, addresses)
        }
        HandleMsg::CreateViewingKey { entropy, .. } => create_viewing_key(deps, &env, entropy),
        HandleMsg::SetViewingKey { key, .. } => set_viewing_key(deps, &env, key),
        HandleMsg::Receive { amount, msg, .. } => match msg.inner {
            ReceiveMsg::Deposit { to } => deposit(deps, &env, to, amount),
            ReceiveMsg::BatchDeposit(deposits) => batch_deposit(deps, &env, deposits, amount),
        },
        HandleMsg::Features(m) => match m {
            FeatureToggleHandleMsg::Pause { features } => {
                FeatureToggle::handle_pause(deps, &env, features)
            }
            FeatureToggleHandleMsg::Unpause { features } => {
                FeatureToggle::handle_unpause(deps, &env, features)
            }
            FeatureToggleHandleMsg::SetPauser { address } => set_pauser(deps, &env, address),
            FeatureToggleHandleMsg::RemovePauser { address } => remove_pauser(deps, &env, address),
        },
        HandleMsg::RevokePermit { permit_name, .. } => revoke_permit(deps, &env, permit_name),
        HandleMsg::ChangeConfig {
            admin,
            unbonding_period,
        } => change_config(deps, &env, admin, unbonding_period),
    };

    let claim_msg = AutoClaims::claim_next_pending_msg(&mut deps.storage, &env)?;
    if let Some(claim_messages) = claim_msg {
        if let Ok(HandleResponse {
            ref mut messages, ..
        }) = result
        {
            messages.extend(claim_messages)
        }
    }

    result
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::NumOfPendingClaims {} => query_num_of_pending_claims(deps),
        QueryMsg::TotalBalances {} => query_total_balances(deps),
        QueryMsg::WithPermit { permit, query } => permit_queries(deps, permit, query),
        QueryMsg::Features(m) => match m {
            FeatureToggleQueryMsg::Status { features } => {
                FeatureToggle::query_status(deps, features)
            }
            FeatureToggleQueryMsg::IsPauser { address } => {
                FeatureToggle::query_is_pauser(deps, address)
            }
        },
        _ => authenticated_queries(deps, msg),
    }
}

pub fn authenticated_queries<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> QueryResult {
    let (address, key) = msg.get_validation_params();
    ViewingKey::check(&deps.storage, address, &key)?;

    match msg {
        QueryMsg::Balance { address, .. } => query_balance(deps, address),
        _ => panic!("This should never happen"),
    }
}

fn permit_queries<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    permit: Permit,
    query: QueryWithPermit,
) -> Result<Binary, StdError> {
    // Validate permit content
    let self_addr = Config::get_unchecked(&deps.storage)?.self_contract_addr;

    let account = validate(deps, PREFIX_REVOKED_PERMITS, &permit, self_addr, None)?;
    let account = HumanAddr(account);

    // Permit validated! We can now execute the query.
    match query {
        QueryWithPermit::Balance {} => {
            if !permit.check_permission(&TokenPermissions::Balance) {
                return Err(StdError::generic_err(format!(
                    "No permission to query balance, got permissions {:?}",
                    permit.params.permissions
                )));
            }

            query_balance(deps, account)
        }
    }
}

fn deposit_impl<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    to: HumanAddr,
    amount: u128,
) -> StdResult<()> {
    // check that the destination address is valid
    let _canonical = deps.api.canonical_address(&to).map_err(|_| {
        StdError::generic_err(format!("Can not deposit to {}. Not a valid address", to))
    })?;

    let mut user_balance = Balances::load(&deps.storage, &to)?.unwrap_or_default();
    user_balance.staked += amount;
    user_balance.save(&mut deps.storage, &to)?;

    Ok(())
}

fn deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    to: HumanAddr,
    amount: Uint128,
) -> HandleResult {
    batch_deposit(deps, env, vec![Deposit::new(to, amount)], amount)
}

fn batch_deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    deposits: Vec<Deposit>,
    amount: Uint128,
) -> HandleResult {
    FeatureToggle::require_not_paused(&deps.storage, vec![Features::Deposit])?;

    let config = Config::get_unchecked(&deps.storage)?;
    if env.message.sender != config.token.address {
        return Err(StdError::generic_err(format!(
            "this token is not supported. Supported: {}, got: {}",
            config.token.address, env.message.sender
        )));
    }

    // Check that the deposits reach the right sum
    let mut sum = amount.u128();
    let sum_err = || {
        StdError::generic_err("The sum of deposits to user accounts does not add up to the amount of LGND sent to the platform")
    };
    for deposit in &deposits {
        sum = sum.checked_sub(deposit.amount.u128()).ok_or_else(sum_err)?;
    }
    if sum != 0 {
        return Err(sum_err());
    }

    for deposit in deposits {
        deposit_impl(deps, deposit.to, deposit.amount.u128())?;
    }

    TotalBalances::handle_balance_change(&mut deps.storage, BalanceChange::Deposit, amount.u128())?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Deposit {
            status: ResponseStatus::Success,
        })?),
    })
}

fn redeem<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    amount: Option<u128>,
) -> HandleResult {
    FeatureToggle::require_not_paused(&deps.storage, vec![Features::Redeem])?;

    let user_address = env.message.sender.clone();
    let mut user_balance = Balances::load(&deps.storage, &user_address)?.unwrap_or_default();
    let amount = amount.unwrap_or(user_balance.staked);
    if amount > user_balance.staked {
        return Err(StdError::generic_err(format!(
            "insufficient staked funds to redeem: balance={}, required={}",
            user_balance.staked, amount,
        )));
    }
    user_balance.pending_redeem.refresh(env); // This refresh is important, because it should prevent auto-claim DoS

    let config = Config::get_unchecked(&deps.storage)?;
    AutoClaims::new_unbonding(
        &mut deps.storage,
        env,
        &config,
        user_address.clone(),
        &mut user_balance,
        amount,
    )?;

    user_balance.staked -= amount;
    user_balance.save(&mut deps.storage, &user_address)?;

    TotalBalances::handle_balance_change(&mut deps.storage, BalanceChange::Redeem, amount)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Redeem {
            status: ResponseStatus::Success,
        })?),
    })
}

fn claim<S: Storage, A: Api, Q: Querier>(deps: &mut Extern<S, A, Q>, env: &Env) -> HandleResult {
    FeatureToggle::require_not_paused(&deps.storage, vec![Features::Claim])?;

    if let Some(messages) = do_claim(&mut deps.storage, env, &env.message.sender)? {
        return Ok(HandleResponse {
            messages,
            log: vec![],
            data: Some(to_binary(&HandleAnswer::ClaimRedeemed {
                status: ResponseStatus::Success,
            })?),
        });
    }

    Err(StdError::generic_err("nothing to claim"))
}

fn send_from_platform<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    contract_addr: HumanAddr,
    amount: Option<u128>,
    memo: Option<String>,
    msg: Binary,
) -> HandleResult {
    FeatureToggle::require_not_paused(&deps.storage, vec![Features::SendFromPlatform])?;
    ReceivingContracts::require_receiving(&deps.storage, &contract_addr)?;

    let mut user_balance = Balances::load(&deps.storage, &env.message.sender)?.unwrap_or_default();
    let amount = amount.unwrap_or(user_balance.staked);

    if amount > user_balance.staked {
        return Err(StdError::generic_err(format!(
            "insufficient staked funds to send: balance={}, required={}",
            user_balance.staked, amount,
        )));
    }

    user_balance.staked -= amount;
    user_balance.save(&mut deps.storage, &env.message.sender)?;

    TotalBalances::handle_balance_change(&mut deps.storage, BalanceChange::Send, amount)?;

    let config = Config::get_unchecked(&deps.storage)?;
    let inner_msg = to_binary(&PlatformApi::ReceiveFromPlatform {
        from: env.message.sender.clone(),
        msg,
    })?;
    let send_msg = snip20::send_msg(
        contract_addr,
        Uint128(amount),
        Some(inner_msg),
        memo,
        None,
        RESPONSE_BLOCK_SIZE,
        config.token.hash,
        config.token.address,
    )?;

    Ok(HandleResponse {
        messages: vec![send_msg],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SendFromPlatform {
            status: ResponseStatus::Success,
        })?),
    })
}

fn add_receiving_contracts<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    addresses: Vec<HumanAddr>,
) -> HandleResult {
    Config::get_unchecked(&deps.storage)?.require_admin(env)?;

    ReceivingContracts::set_multiple(&mut deps.storage, addresses)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddReceivingContracts {
            status: ResponseStatus::Success,
        })?),
    })
}

fn remove_receiving_contracts<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    addresses: Vec<HumanAddr>,
) -> HandleResult {
    Config::get_unchecked(&deps.storage)?.require_admin(env)?;

    ReceivingContracts::remove_multiple(&mut deps.storage, addresses);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RemoveReceivingContracts {
            status: ResponseStatus::Success,
        })?),
    })
}

fn create_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    entropy: String,
) -> HandleResult {
    let key = ViewingKey::create(
        &mut deps.storage,
        env,
        &env.message.sender,
        entropy.as_bytes(),
    );

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CreateViewingKey { key })?),
    })
}

fn set_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    key: String,
) -> StdResult<HandleResponse> {
    ViewingKey::set(&mut deps.storage, &env.message.sender, &key);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetViewingKey { status: Success })?),
    })
}

fn set_pauser<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    address: HumanAddr,
) -> HandleResult {
    Config::get_unchecked(&deps.storage)?.require_admin(env)?;

    FeatureToggle::handle_set_pauser(deps, env, address)
}

fn remove_pauser<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    address: HumanAddr,
) -> HandleResult {
    Config::get_unchecked(&deps.storage)?.require_admin(env)?;

    FeatureToggle::handle_remove_pauser(deps, env, address)
}

fn revoke_permit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    permit_name: String,
) -> StdResult<HandleResponse> {
    RevokedPermits::revoke_permit(
        &mut deps.storage,
        PREFIX_REVOKED_PERMITS,
        &env.message.sender,
        &permit_name,
    );

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RevokePermit { status: Success })?),
    })
}

fn change_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    admin: Option<HumanAddr>,
    unbonding_period: Option<u64>,
) -> StdResult<HandleResponse> {
    let mut config = Config::get_unchecked(&deps.storage)?;
    config.require_admin(env)?;

    if let Some(admin) = admin {
        config.admin = admin;
    }

    if let Some(unbonding_period) = unbonding_period {
        config.unbonding_period = unbonding_period
    }

    config.save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ChangeConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

fn query_config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let config = Config::get_unchecked(&deps.storage)?;
    let result = to_binary(&QueryAnswer::Config(config))?;
    Ok(result)
}

fn query_num_of_pending_claims<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> QueryResult {
    let len = AutoClaims::len(&deps.storage)?;
    let result = to_binary(&QueryAnswer::NumOfPendingClaims(Uint128::from(len as u64)))?;
    Ok(result)
}

fn query_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: HumanAddr,
) -> QueryResult {
    // Assuming authentication occurs in the calling function
    let balance = Balances::load(&deps.storage, &account)?.unwrap_or_default();
    let result = to_binary(&QueryAnswer::Balance(balance.into()))?;
    Ok(result)
}

fn query_total_balances<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let total_balances = TotalBalances::load(&deps.storage)?.unwrap_or_default();
    let result = to_binary(&QueryAnswer::TotalBalances(
        total_balances.to_query_result().into(),
    ))?;

    Ok(result)
}

pub fn do_claim<S: Storage>(
    storage: &mut S,
    env: &Env,
    account: &HumanAddr,
) -> StdResult<Option<Vec<CosmosMsg>>> {
    let config = Config::get_unchecked(storage)?;
    let mut user_balance = Balances::load(storage, account)?.unwrap_or_default();
    user_balance.pending_redeem.refresh(env);

    let amount = user_balance.pending_redeem.claimable;
    if amount == 0 {
        return Ok(None);
    }

    user_balance.pending_redeem.claimable -= amount;
    user_balance.save(storage, account)?;

    TotalBalances::handle_balance_change(storage, BalanceChange::Claim, amount)?;

    let amount = Uint128::from(amount);
    let messages = vec![
        snip20::redeem_msg(
            amount,
            Some(config.native_token_denom.clone()),
            None,
            RESPONSE_BLOCK_SIZE,
            config.token.hash,
            config.token.address,
        )?,
        CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address.clone(),
            to_address: account.into(),
            amount: vec![Coin {
                denom: config.native_token_denom,
                amount,
            }],
        }),
    ];

    Ok(Some(messages))
}
