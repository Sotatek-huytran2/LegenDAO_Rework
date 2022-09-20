use cosmwasm_std::{log, Api, Env, Extern, HandleResponse, Querier, StdResult, Storage};

use crate::state::Config;

pub fn handle_set_quest_password<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    mut config: Config,
    password: String,
) -> StdResult<HandleResponse> {
    config.assert_quest_contract(&env.message.sender)?;

    config.quest_password = Some(password);

    config.save(&mut deps.storage)?;

    Ok(HandleResponse {
        log: vec![log("changed", "password")],
        ..Default::default()
    })
}
