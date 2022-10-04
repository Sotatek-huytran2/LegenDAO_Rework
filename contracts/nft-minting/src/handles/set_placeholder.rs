use crate::state::config_read;
use crate::types::hidden_token::{hidden_token, HiddenToken};
use cosmwasm_std::{Api, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage};

pub fn set_placeholder<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    token_uri: String,
) -> StdResult<HandleResponse> {
    let config = config_read(&deps.storage).may_load()?.unwrap();
    if config.owner != env.message.sender {
        return Err(StdError::generic_err(
            "Cannot set placeholder from non-admin address",
        ));
    }

    let placeholder = HiddenToken { token_uri };

    hidden_token(&mut deps.storage).save(&placeholder)?;

    Ok(HandleResponse::default())
}
