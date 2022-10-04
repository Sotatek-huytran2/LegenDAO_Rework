use crate::msg::RemainingResponse;
use crate::types::custom_rng::NftRng;
use cosmwasm_std::{Api, Extern, Querier, StdResult, Storage};

pub fn query_remaining<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<RemainingResponse> {
    let rng = NftRng::load(&deps.storage)?;

    Ok(RemainingResponse {
        remaining: rng.remaining() as u32,
    })
}
