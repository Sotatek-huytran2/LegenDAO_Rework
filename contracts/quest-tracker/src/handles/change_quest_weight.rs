use cosmwasm_std::{log, Api, Env, Extern, HandleResponse, Querier, StdResult, Storage};

use crate::state::{add_quest_weight, Config};

/// Registers a contract that is allowed to signal quest success
///
pub fn change_quest_weight<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config,
    quest: u8,
    weight: u8,
) -> StdResult<HandleResponse> {
    config.assert_admin(&env.message.sender)?;

    add_quest_weight(&mut deps.storage, quest, weight)?;

    Ok(HandleResponse {
        log: vec![log("change quest contract", "success")],
        ..Default::default()
    })
}
