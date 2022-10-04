use crate::msgs::msg_enable_reveal::enable_reveal_msg;
use crate::state::config;
use cosmwasm_std::{
    log, Api, CosmosMsg, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage,
};

pub fn try_enable_reveal<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let mut state = config(&mut deps.storage).may_load()?.unwrap();
    if state.owner != env.message.sender {
        return Err(StdError::generic_err(
            "Cannot enable reveal from non-admin address",
        ));
    }
    let contract = state.nft_contract.clone();
    state.is_revealed = true;

    config(&mut deps.storage).save(&state)?;

    let reveal_msg = enable_reveal_msg(contract);
    let messages: Vec<CosmosMsg> = vec![reveal_msg];

    Ok(HandleResponse {
        messages,
        log: vec![log("reveal", "enabled")], // plaintext log this
        data: None,
    })
}
