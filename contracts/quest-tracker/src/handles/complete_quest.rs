use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, HumanAddr, Querier, QueryRequest, StdError,
    StdResult, Storage, Uint128, WasmQuery,
};

use crate::state::{get_quest_contract, get_quest_weight, set_quest_status, Config};

use airdrop::QueryMsg::GetAccountInfo;
use airdrop::{AccountInfoResponse, Deposit, LgndReceiveMsg};

pub fn complete_quest<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config,
    address: HumanAddr,
) -> StdResult<HandleResponse> {
    let quest = get_quest_contract(&deps.storage, &env.message.sender);

    let weight = get_quest_weight(&deps.storage, quest);

    let airdrop_contract = if let Some(contract) = config.airdrop_contract {
        Ok(contract)
    } else {
        Err(StdError::generic_err(
            "Cannot complete quest before airdrop contract is set up",
        ))
    }?;

    // get base amount from airdrop contract
    let query_msg = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: airdrop_contract.address,
        callback_code_hash: airdrop_contract.hash,
        msg: to_binary(&GetAccountInfo {
            password: config.password.unwrap(),
            address: address.clone(),
        })?,
    });
    let query_result: AccountInfoResponse = deps.querier.query(&query_msg)?;
    let base_amount = query_result.account.amount.u128();

    set_quest_status(&mut deps.storage, &address, quest)?;

    let mut distribute_msg = vec![];
    if weight > 0 {
        let send_amount = Uint128(base_amount * weight as u128);

        distribute_msg.push(secret_toolkit::snip20::send_msg_with_code_hash(
            config.platform.address,
            Some(config.platform.hash),
            send_amount,
            Some(to_binary(&LgndReceiveMsg::BatchDeposit(vec![Deposit {
                to: address,
                amount: send_amount,
            }]))?),
            Some("complete quest".to_string()),
            None,
            64,
            config.token.hash,
            config.token.address,
        )?);

        // distribute_msg.push(secret_toolkit::snip20::transfer_msg(
        //     address,
        //     Uint128(base_amount * weight as u128),
        //     None,
        //     None,
        //     64,
        //     config.token.hash,
        //     config.token.address,
        // )?)
    }

    Ok(HandleResponse {
        messages: distribute_msg,
        log: vec![],
        data: None,
    })
}
