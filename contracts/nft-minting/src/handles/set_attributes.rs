use crate::handles::utils::check_admin;
use crate::types::token_attributes::{set_nft_attributes, InputTokenAttributes};
use cosmwasm_std::{Api, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage};

pub fn try_set_attributes<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    attributes: Vec<InputTokenAttributes>,
) -> StdResult<HandleResponse> {
    check_admin(deps, &env)?;

    for attr in attributes {
        let token_id = attr
            .token_id
            .parse::<u64>()
            .map_err(|_| StdError::generic_err("Failed to parse Token ID"))?;
        set_nft_attributes(&mut deps.storage, token_id, &attr.attributes)?;
    }

    Ok(HandleResponse::default())
}
