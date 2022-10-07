use crate::msg::{RemainingResponse, CapAmountResponse};
use crate::types::custom_rng::NftRng;
use cosmwasm_std::{Api, Extern, Querier, StdResult, Storage, QueryResult, to_binary};
use crate::state::{
    Config, config_read
};


pub fn query_remaining<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<RemainingResponse> {
    let rng = NftRng::load(&deps.storage)?;

    Ok(RemainingResponse {
        remaining: rng.remaining() as u32,
    })
}


pub fn query_cap<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<CapAmountResponse> {
    let cap = config_read(&deps.storage).may_load()?.unwrap();

    Ok(CapAmountResponse {
        cap_amount: (cap.cap_amount).unwrap(),
    })
}
