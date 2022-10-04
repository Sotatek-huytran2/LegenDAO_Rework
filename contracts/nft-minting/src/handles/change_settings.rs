use crate::handles::utils::check_admin;
use crate::state::{config, config_read, OptionalConfig};
use cosmwasm_std::{Api, Env, Extern, HandleResponse, Querier, StdResult, Storage};

pub fn change_settings<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    settings: OptionalConfig,
) -> StdResult<HandleResponse> {
    check_admin(deps, &env)?;

    let mut state = config_read(&deps.storage).may_load()?.unwrap();

    if settings.minting_enabled.is_some() {
        state.minting_enabled = settings.minting_enabled.unwrap();
    }

    if settings.price.is_some() {
        state.price = settings.price.unwrap()
    }

    if settings.platform.is_some() {
        state.platform = settings.platform.unwrap();
    }

    if settings.only_platform.is_some() {
        state.only_platform = settings.only_platform.unwrap();
    }

    if settings.is_revealed.is_some() {
        state.is_revealed = settings.is_revealed.unwrap();
    }

    if settings.max_batch_mint.is_some() {
        state.max_batch_mint = settings.max_batch_mint.unwrap();
    }

    if settings.nft_contract.is_some() {
        state.nft_contract = settings.nft_contract.unwrap();
    }

    if settings.nft_count.is_some() {
        state.nft_count = settings.nft_count.unwrap();
    }

    if settings.owner.is_some() {
        state.owner = settings.owner.unwrap();
    }

    config(&mut deps.storage).save(&state)?;

    Ok(HandleResponse::default())
}
