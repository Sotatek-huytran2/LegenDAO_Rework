use cosmwasm_std::{
    plaintext_log, to_binary, Api, Env, Extern, HandleResponse, Querier, StdError, StdResult,
    Storage, Uint128,
};

use crate::state::{
    add_allocation_for_address, get_allocation_for_account, is_claimed, set_claimed, Config,
};
use crate::types::airdrop::AirdropClaimSubmit;
use crate::types::deposit::{Deposit, LgndReceiveMsg};

pub fn confirm_airdrop<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config,
    airdrops: Vec<AirdropClaimSubmit>,
) -> StdResult<HandleResponse> {
    config.assert_confirmer(&env.message.sender)?;

    let mut deposits = vec![];
    let mut logs = vec![];
    let mut sum = 0_u128;

    for drop in airdrops {
        if is_claimed(&deps.storage, &drop.address) {
            continue;
        }

        sum = sum.checked_add(drop.amount.u128()).ok_or_else(|| {
            let msg = format!("Airdrop to {} would exceed u128::MAX", drop.address);
            StdError::generic_err(msg)
        })?;

        set_claimed(&mut deps.storage, &drop.address)?;

        // add total amount to tracker
        let cur_amount = get_allocation_for_account(&deps.storage, &drop.to);
        add_allocation_for_address(&mut deps.storage, &drop.to, cur_amount + drop.amount.u128())?;

        logs.push(plaintext_log("airdropped_to", &drop.address));
        deposits.push(Deposit::new(drop.to, drop.amount));
    }

    let distribute_msg = secret_toolkit::snip20::send_msg_with_code_hash(
        config.platform.address,
        Some(config.platform.hash),
        Uint128(sum),
        Some(to_binary(&LgndReceiveMsg::BatchDeposit(deposits))?),
        Some("LGND Airdrop".to_string()),
        None,
        64,
        config.token.hash,
        config.token.address,
    )?;

    Ok(HandleResponse {
        messages: vec![distribute_msg],
        log: logs,
        data: None,
    })
}
