use crate::state::config;
use crate::types::minting_level::MintingLevel;
use cosmwasm_std::{Api, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage};

pub fn set_minting_level<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    new_level: MintingLevel,
    cap_amount: Option<u16>,
) -> StdResult<HandleResponse> {
    let mut state = config(&mut deps.storage).may_load()?.unwrap();
    if state.owner != env.message.sender {
        return Err(StdError::generic_err(
            "Cannot set attributes from non-admin address",
        ));
    }

    state.minting_enabled = new_level;
    state.cap_amount = cap_amount;

    config(&mut deps.storage).save(&state)?;

    Ok(HandleResponse::default())
}
