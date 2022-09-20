use crate::queries::{AccountInfo, AccountInfoResponse};
use crate::state::{get_allocation_for_account, Config};
use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdResult, Storage, Uint128};

pub fn query_get_account_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    password: String,
    address: HumanAddr,
) -> StdResult<AccountInfoResponse> {
    let config = Config::load(&deps.storage)?;
    config.assert_password(&password)?;

    let account = AccountInfo {
        amount: Uint128(get_allocation_for_account(&deps.storage, &address)),
        address,
    };

    // let accounts: Vec<AccountInfo> = addresses
    //     .iter()
    //     .map(|acc| AccountInfo {
    //         amount: get_allocation_for_account(&deps.storage, acc),
    //         address: acc.clone(),
    //     })
    //     .collect();

    Ok(AccountInfoResponse { account })
}
