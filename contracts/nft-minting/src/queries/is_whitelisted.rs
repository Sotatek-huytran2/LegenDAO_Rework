use crate::msg::IsWhitelistedResponse;
use crate::types::whitelist::get_whitelist;
use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdResult, Storage};

pub fn query_is_whitelisted<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> StdResult<IsWhitelistedResponse> {
    let allowed_to_mint = get_whitelist(&deps.storage, &address);

    let response = if allowed_to_mint == 0 {
        IsWhitelistedResponse {
            is_whitelisted: false,
            amount: None,
        }
    } else {
        IsWhitelistedResponse {
            is_whitelisted: true,
            amount: Some(allowed_to_mint),
        }
    };

    Ok(response)
}
