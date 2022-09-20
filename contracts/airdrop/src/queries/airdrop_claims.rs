use cosmwasm_std::{Api, Extern, Querier, StdResult, Storage};

use crate::state::is_claimed;

use super::AirdropClaimResponse;

pub fn query_airdrop_claims<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: String,
) -> StdResult<AirdropClaimResponse> {
    let result = is_claimed(&deps.storage, &address);

    Ok(AirdropClaimResponse { claimed: result })
}
