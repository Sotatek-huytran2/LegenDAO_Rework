use cosmwasm_std::{log, Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdResult, Storage};

use crate::state::{add_quest_contract, Config};

/// Registers a contract that is allowed to signal quest success
///
pub fn register_quest_contract<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config,
    address: HumanAddr,
    quest: u8,
) -> StdResult<HandleResponse> {
    config.assert_admin(&env.message.sender)?;

    add_quest_contract(&mut deps.storage, &address, quest)?;

    Ok(HandleResponse {
        log: vec![log("register quest contract", "success")],
        ..Default::default()
    })
}
