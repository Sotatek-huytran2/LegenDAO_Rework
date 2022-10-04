use crate::state::config_read;
use crate::types::whitelist::change_allocation;
use cosmwasm_std::{
    Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage,
};

pub fn remove_whitelist<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    addresses: Vec<HumanAddr>,
) -> StdResult<HandleResponse> {
    let config = config_read(&deps.storage).may_load()?.unwrap();
    if config.owner != env.message.sender {
        return Err(StdError::generic_err(
            "Cannot remove from whitelist from non-admin address",
        ));
    }

    for address in addresses {
        change_allocation(&mut deps.storage, &address, 0)?;
    }

    Ok(HandleResponse::default())
}
