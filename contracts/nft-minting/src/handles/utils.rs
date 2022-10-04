use crate::msg::{MintPrice, Token};
use crate::state::config_read;
use cosmwasm_std::{Api, Env, Extern, Querier, StdError, StdResult, Storage, Uint128};
use secret_toolkit::utils::types::Contract;

pub fn check_paid_for_mint(
    // list of
    configured_prices: &[MintPrice],
    // snip or native token sent
    paid_with_token: &Token,
    // how much of the token was sent
    paid: Uint128,
    // amount of tokens to buy
    amount: Option<u8>,
    is_whitelist: bool,
) -> StdResult<()> {
    let price = configured_prices.iter().find(|p| match &p.token {
        Token::Snip20(Contract { address, .. }) => {
            if let Token::Snip20(contract) = paid_with_token {
                if &contract.address == address {
                    return true;
                }
            }
            false
        }
        Token::Native(s) => {
            if let Token::Native(paid_string) = paid_with_token {
                paid_string == s
            } else {
                false
            }
        }
    });

    if price.is_none() {
        return Err(StdError::generic_err(
            "Tried to mint with unsupported token",
        ));
    }

    let total_cost = if is_whitelist {
        Uint128(price.unwrap().whitelist_price.u128() * amount.unwrap_or(1) as u128)
    } else {
        Uint128(price.unwrap().price.u128() * amount.unwrap_or(1) as u128)
    };

    if total_cost != paid {
        return Err(StdError::generic_err(format!(
            "Failed to mint; Wrong amount of coins was sent. Got: {}, expected: {}",
            paid, total_cost
        )));
    }

    Ok(())
}

pub fn check_admin<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: &Env,
) -> StdResult<()> {
    let config = config_read(&deps.storage).may_load()?.unwrap();
    if config.owner != env.message.sender {
        return Err(StdError::generic_err(
            "Cannot perform this action from non-admin address",
        ));
    }

    Ok(())
}
