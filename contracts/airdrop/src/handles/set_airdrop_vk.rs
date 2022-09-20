use cosmwasm_std::{log, Env, HandleResponse, StdResult};

use secret_toolkit::snip20::set_viewing_key_msg;

use crate::state::Config;

pub fn set_airdrop_vk(env: Env, config: Config, viewing_key: String) -> StdResult<HandleResponse> {
    config.assert_admin(&env.message.sender)?;

    let msg = set_viewing_key_msg(
        viewing_key,
        None,
        256,
        config.token.hash,
        config.token.address,
    )?;

    Ok(HandleResponse {
        log: vec![log("changed", "viewing key")],
        messages: vec![msg],
        data: None,
    })
}
