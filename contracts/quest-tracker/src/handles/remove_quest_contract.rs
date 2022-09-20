use cosmwasm_std::{log, Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdResult, Storage};

use crate::state::{add_quest_contract, Config};

/// Remove a contract from quest signaling
pub fn remove_quest_contract<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config,
    address: HumanAddr,
) -> StdResult<HandleResponse> {
    config.assert_admin(&env.message.sender)?;

    add_quest_contract(&mut deps.storage, &address, 0u8)?;

    Ok(HandleResponse {
        log: vec![log("change quest contract", "success")],
        ..Default::default()
    })
}
