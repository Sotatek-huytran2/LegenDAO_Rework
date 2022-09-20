use cosmwasm_std::{log, Api, Env, Extern, HandleResponse, Querier, StdResult, Storage};

use crate::state::Config;

use super::NewConfig;

pub fn change_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    mut config: Config,
    new_config: NewConfig,
) -> StdResult<HandleResponse> {
    config.assert_admin(&env.message.sender)?;

    config.admin = new_config.admin.unwrap_or(config.admin);
    config.confirmer = new_config.confirmer.unwrap_or(config.confirmer);
    config.platform = new_config.platform.unwrap_or(config.platform);
    config.token = new_config.token.unwrap_or(config.token);

    // reset quest contract
    if let Some(quest_contract) = new_config.quest_contract {
        config.quest_contract = Some(quest_contract);
        config.quest_password = None;
    }

    config.save(&mut deps.storage)?;

    Ok(HandleResponse {
        log: vec![log("changed", "config")],
        ..Default::default()
    })
}
