use cosmwasm_std::{
    log, to_binary, Api, CosmosMsg, Env, Extern, HandleResponse, Querier, StdResult, Storage,
    WasmMsg,
};

use airdrop::HandleMsg::SetQuestPassword;

use crate::state::Config;
use crate::types::secret_contract::SecretContract;

/// Registers a contract that is allowed to signal quest success
///
pub fn register_airdrop_contract<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    mut config: Config,
    contract: SecretContract,
) -> StdResult<HandleResponse> {
    config.assert_admin(&env.message.sender)?;

    // todo: generate this
    config.password = Some("hello".to_string());

    config.airdrop_contract = Some(contract.clone());

    config.save(&mut deps.storage)?;

    let msg = to_binary(&SetQuestPassword {
        password: config.password.unwrap(),
    })?;

    let set_password_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract.address,
        callback_code_hash: contract.hash,
        msg,
        send: vec![],
    });

    Ok(HandleResponse {
        messages: vec![set_password_msg],
        log: vec![log("register quest contract", "success")],
        ..Default::default()
    })
}
