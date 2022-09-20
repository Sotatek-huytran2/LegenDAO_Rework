#![cfg(test)]

use std::any::Any;

use cosmwasm_std::testing::{
    mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
};
use cosmwasm_std::{
    from_binary, BlockInfo, ContractInfo, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
    MessageInfo, StdResult, Uint128,
};
use secret_toolkit::serialization::Base64JsonOf;
use secret_toolkit::utils::feature_toggle::FeatureToggleHandleMsg;
use secret_toolkit::utils::types::Contract;

use crate::contract::{handle, init};
use crate::msg::HandleMsg::{Receive, SetViewingKey};
use crate::msg::{HandleAnswer, HandleMsg, InitMsg, ReceiveFromPlatformMsg, ReceiveMsg};
use crate::state::{Features, ScheduleUnit};

pub fn extract_answer(hr: HandleResponse) -> StdResult<HandleAnswer> {
    let HandleResponse { data, .. } = hr;
    from_binary(&data.unwrap())
}

pub fn extract_messages(hr: HandleResponse) -> Vec<CosmosMsg> {
    let HandleResponse { messages, .. } = hr;
    messages
}

pub fn mock_env_with_height<U: Into<HumanAddr>>(sender: U, height: u64) -> Env {
    Env {
        block: BlockInfo {
            height,
            time: 12_345,
            chain_id: "cosmos-testnet-14002".to_string(),
        },
        message: MessageInfo {
            sender: sender.into(),
            sent_funds: vec![],
        },
        contract: ContractInfo {
            address: HumanAddr::from(MOCK_CONTRACT_ADDR),
        },
        contract_key: Some("".to_string()),
        contract_code_hash: "".to_string(),
    }
}

pub fn extract_generic_error_msg<T: Any>(error: StdResult<T>) -> String {
    match error {
        Ok(_) => {
            panic!("Handle Response is not an error")
        }
        Err(err) => match err {
            cosmwasm_std::StdError::GenericErr { msg, .. } => msg,
            _ => panic!("Error is not generic"),
        },
    }
}

pub fn init_helper(
    subscribers: Option<Vec<Contract>>,
    multiplier_contracts: Option<Vec<HumanAddr>>,
    inflation_schedule: Option<Vec<ScheduleUnit>>,
    max_multiplier: Option<u32>,
) -> StdResult<Extern<MockStorage, MockApi, MockQuerier>> {
    let mut deps = mock_dependencies(20, &[]);
    let env = mock_env_with_height("admin", 0);

    let init_msg = InitMsg {
        token: Contract {
            address: HumanAddr::from("token"),
            hash: "".to_string(),
        },
        platform: Contract {
            address: HumanAddr::from("platform"),
            hash: "".to_string(),
        },
        viewing_key: "vk".to_string(),
        prng_seed: Default::default(),
        subscribers,
        inflation_schedule: inflation_schedule.unwrap_or(vec![]),
        max_multiplier: max_multiplier.map(|x| Uint128(x as u128)),
        multiplier_contracts,
    };

    init(&mut deps, env, init_msg)?;
    Ok(deps)
}

pub fn set_viewing_key_helper(
    deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
    from: &str,
    viewing_key: &str,
) -> StdResult<HandleAnswer> {
    let result = handle(
        deps,
        mock_env(HumanAddr::from(from), &[]),
        SetViewingKey {
            key: viewing_key.to_string(),
            padding: None,
        },
    )?;

    extract_answer(result)
}

pub fn add_multiplier_contracts_helper(
    deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
    from: &str,
    contracts: Vec<HumanAddr>,
) -> StdResult<HandleAnswer> {
    let result = handle(
        deps,
        mock_env(HumanAddr::from(from), &[]),
        HandleMsg::AddMultiplierContracts { contracts },
    )?;

    extract_answer(result)
}

pub fn remove_multiplier_contracts_helper(
    deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
    from: &str,
    contracts: Vec<HumanAddr>,
) -> StdResult<HandleAnswer> {
    let result = handle(
        deps,
        mock_env(HumanAddr::from(from), &[]),
        HandleMsg::RemoveMultiplierContracts { contracts },
    )?;

    extract_answer(result)
}

pub fn add_subscriber_contracts_helper(
    deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
    from: &str,
    contracts: Vec<Contract>,
) -> StdResult<HandleAnswer> {
    let result = handle(
        deps,
        mock_env(HumanAddr::from(from), &[]),
        HandleMsg::AddSubs { contracts },
    )?;

    extract_answer(result)
}

pub fn remove_subscriber_contracts_helper(
    deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
    from: &str,
    contracts: Vec<HumanAddr>,
) -> StdResult<HandleAnswer> {
    let result = handle(
        deps,
        mock_env(HumanAddr::from(from), &[]),
        HandleMsg::RemoveSubs { contracts },
    )?;

    extract_answer(result)
}

pub fn deposit_helper(
    deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
    depositor: &str,
    amount: u128,
    block_height: Option<u64>,
) -> StdResult<(Vec<CosmosMsg>, HandleAnswer)> {
    let result = handle(
        deps,
        mock_env_with_height(HumanAddr::from("token"), block_height.unwrap_or(0)),
        Receive {
            sender: HumanAddr::from("not-used"),
            from: HumanAddr::from("platform"),
            amount: Uint128::from(amount),
            msg: Base64JsonOf::from(ReceiveMsg::ReceiveFromPlatform {
                from: HumanAddr::from(depositor),
                msg: Base64JsonOf::from(ReceiveFromPlatformMsg::Deposit {}),
            }),
        },
    )?;

    Ok((extract_messages(result.clone()), extract_answer(result)?))
}

pub fn withdraw_helper(
    deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
    redeemer: &str,
    amount: u128,
    block_height: Option<u64>,
) -> StdResult<(Vec<CosmosMsg>, HandleAnswer)> {
    let result = handle(
        deps,
        mock_env_with_height(HumanAddr::from(redeemer), block_height.unwrap_or(0)),
        HandleMsg::Withdraw {
            amount: Some(Uint128::from(amount)),
        },
    )?;

    Ok((extract_messages(result.clone()), extract_answer(result)?))
}

pub fn apply_multiplier_helper(
    deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
    sending_contract: &str,
    multiplier: u32,
    block_height: Option<u64>,
    item_id: &str,
    to: &str,
) -> StdResult<(Vec<CosmosMsg>, HandleAnswer)> {
    let result = handle(
        deps,
        mock_env_with_height(HumanAddr::from(sending_contract), block_height.unwrap_or(0)),
        HandleMsg::ApplyMultiplier {
            to: HumanAddr::from(to),
            multiplier,
            item_id: item_id.to_string(),
        },
    )?;

    Ok((extract_messages(result.clone()), extract_answer(result)?))
}

pub fn drop_multiplier_helper(
    deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
    sending_contract: &str,
    block_height: Option<u64>,
    item_id: &str,
    from: &str,
) -> StdResult<(Vec<CosmosMsg>, HandleAnswer)> {
    let result = handle(
        deps,
        mock_env_with_height(HumanAddr::from(sending_contract), block_height.unwrap_or(0)),
        HandleMsg::DropMultiplier {
            from: HumanAddr::from(from),
            item_id: item_id.to_string(),
        },
    )?;

    Ok((extract_messages(result.clone()), extract_answer(result)?))
}

pub fn emergency_withdraw_helper(
    deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
    withdrawer: &str,
) -> StdResult<(Vec<CosmosMsg>, HandleAnswer)> {
    let result = handle(
        deps,
        mock_env(HumanAddr::from(withdrawer), &[]),
        HandleMsg::EmergencyWithdraw {},
    )?;

    Ok((extract_messages(result.clone()), extract_answer(result)?))
}

pub fn emergency_withdraw_skip_platform_helper(
    deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
    withdrawer: &str,
) -> StdResult<(Vec<CosmosMsg>, HandleAnswer)> {
    let result = handle(
        deps,
        mock_env(HumanAddr::from(withdrawer), &[]),
        HandleMsg::EmergencyWithdrawSkipPlatform {},
    )?;

    Ok((extract_messages(result.clone()), extract_answer(result)?))
}

pub fn unpause_feature_helper(
    deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
    feature: Features,
) -> StdResult<()> {
    handle(
        deps,
        mock_env(HumanAddr::from("admin"), &[]),
        HandleMsg::Features(FeatureToggleHandleMsg::Unpause {
            features: vec![feature],
        }),
    )?;

    Ok(())
}

pub fn change_max_mul_config_helper(
    deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
    new_max_multiplier: u32,
) -> StdResult<(Vec<CosmosMsg>, HandleAnswer)> {
    let result = handle(
        deps,
        mock_env(HumanAddr::from("admin"), &[]),
        HandleMsg::ChangeConfig {
            admin: None,
            platform: None,
            token_vk: None,
            inflation: None,
            max_multiplier: Some(Uint128::from(new_max_multiplier as u128)),
        },
    )?;

    Ok((extract_messages(result.clone()), extract_answer(result)?))
}
