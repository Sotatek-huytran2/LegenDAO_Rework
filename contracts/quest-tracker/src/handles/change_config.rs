use cosmwasm_std::{
    log, to_binary, Api, CosmosMsg, Env, Extern, HandleResponse, Querier, StdResult, Storage,
    WasmMsg,
};

use airdrop::HandleMsg::SetQuestPassword;

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
    config.token = new_config.token.unwrap_or(config.token);
    config.platform = new_config.platform.unwrap_or(config.platform);

    let mut messages = vec![];

    // reset airdrop contract
    if let Some(airdrop_contract) = new_config.airdrop_contract {
        config.airdrop_contract = Some(airdrop_contract.clone());

        // todo: generate password
        config.password = Some("hello".to_string());

        let msg = to_binary(&SetQuestPassword {
            password: config.password.clone().unwrap(),
        })?;

        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: airdrop_contract.address,
            callback_code_hash: airdrop_contract.hash,
            msg,
            send: vec![],
        }));
    }

    config.save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages,
        log: vec![log("changed", "config")],
        ..Default::default()
    })
}
