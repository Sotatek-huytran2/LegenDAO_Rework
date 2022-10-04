use crate::msg::WhitelistAddress;
use crate::state::config_read;
use crate::types::whitelist::add_to_whitelist;
use cosmwasm_std::{Api, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage};

pub fn add_whitelist<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    addresses: Vec<WhitelistAddress>,
) -> StdResult<HandleResponse> {
    let config = config_read(&deps.storage).may_load()?.unwrap();
    if config.owner != env.message.sender {
        return Err(StdError::generic_err(
            "Cannot add to whitelist from non-admin address",
        ));
    }

    for address in addresses {
        add_to_whitelist(&mut deps.storage, &address.address, address.amount)?;
    }

    Ok(HandleResponse::default())
}
