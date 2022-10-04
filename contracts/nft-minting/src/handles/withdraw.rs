use crate::handles::utils::check_admin;
use crate::msg::Token;
use cosmwasm_std::{
    Api, BankMsg, Binary, Coin, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, Querier,
    StdResult, Storage, Uint128,
};
use secret_toolkit::snip20;
use secret_toolkit::utils::types::Contract;

pub(crate) fn withdraw_funds<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    dest: HumanAddr,
    token: Token,
    amount: Uint128,
    snip20_send_msg: Option<Binary>,
) -> StdResult<HandleResponse> {
    check_admin(deps, &env)?;

    let withdraw_msg = match token {
        Token::Snip20(Contract { address, hash }) => vec![snip20::send_msg(
            dest,
            amount,
            snip20_send_msg,
            None,
            None,
            256,
            hash,
            address,
        )?],
        Token::Native(denom) => vec![CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address,
            to_address: dest,
            amount: vec![Coin::new(amount.u128(), &denom)],
        })],
    };

    Ok(HandleResponse {
        messages: withdraw_msg,
        log: vec![],
        data: None,
    })
}
