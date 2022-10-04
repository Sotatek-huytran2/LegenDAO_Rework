use crate::msg::MintingLevelResponse;

use crate::state::config_read;

use cosmwasm_std::{Api, Extern, Querier, StdResult, Storage};

pub fn query_minting_level<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<MintingLevelResponse> {
    let state = config_read(&deps.storage).may_load()?.unwrap();

    Ok(MintingLevelResponse {
        minting_level: state.minting_enabled.to_string(),
    })
}
