use cosmwasm_std::{
    log, Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage,
    Uint128,
};
use secret_toolkit::utils::types::Contract;

use crate::snip721::extension::Extension;
use crate::snip721::metadata::Metadata;
use crate::snip721::snip721_handle_msg::Mint;

use crate::handles::utils::{check_admin, check_paid_for_mint};
use crate::msgs::mint_nft::batch_mint;
use crate::state::{config, config_read, Config};
use crate::types::custom_rng::NftRng;
use crate::types::minting_level::MintingLevel;
use crate::types::token_attributes::{get_nft_attributes, Attributes};
use crate::types::whitelist::{change_allocation, get_whitelist};

use crate::msg::Token;

pub fn try_mint_native<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Option<u8>,
) -> StdResult<HandleResponse> {
    let config = config_read(&deps.storage).may_load()?.unwrap();

    if env.message.sent_funds.len() != 1 {
        return Err(StdError::generic_err(
            "Mint with native coins must contain a single denom of sent funds",
        ));
    }

    let is_whitelist = get_is_whitelist(&config)?;

    if config.max_batch_mint < amount.unwrap_or(1) {
        return Err(StdError::generic_err(format!(
            "Cannot mint more than {} tokens",
            config.max_batch_mint
        )));
    }

    let sent_funds = &env.message.sent_funds[0];

    check_paid_for_mint(
        &config.price,
        &Token::Native(sent_funds.denom.clone()),
        sent_funds.amount,
        amount,
        is_whitelist,
    )?;

    check_cap_amount(&mut deps.storage, &config, amount.unwrap_or(1))?;

    match config.minting_enabled {
        MintingLevel::Whitelist => try_mint_whitelist(deps, env.message.sender, amount),
        MintingLevel::Public => do_mint(deps, env.message.sender, amount),
        MintingLevel::Disabled => Err(StdError::generic_err("Minting not enabled yet")),
        MintingLevel::AdminOnly => Err(StdError::generic_err("Minting not enabled yet")),
    }
}

fn get_is_whitelist(config: &Config) -> StdResult<bool> {
    match config.minting_enabled {
        MintingLevel::Disabled => Err(StdError::generic_err("Minting currently disabled")),
        MintingLevel::AdminOnly => Err(StdError::generic_err("Minting currently disabled")),
        MintingLevel::Whitelist => Ok(true),
        MintingLevel::Public => Ok(false),
    }
}

/// this function assumes that payment has already been handled and just does the minting
fn do_mint<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    owner: HumanAddr,
    amount: Option<u8>,
) -> StdResult<HandleResponse> {
    let config = config_read(&deps.storage).may_load()?.unwrap();

    let contract = config.nft_contract;

    let to_mint = amount.unwrap_or(1);

    let mut messages = vec![];

    let mut mints: Vec<Mint> = vec![];
    let mut tokens_minted: Vec<String> = vec![];

    for _ in 0..to_mint {
        let mut rng = NftRng::load(&deps.storage)?;
        let token_id = rng.next(&mut deps.storage)?;

        rng.save(&mut deps.storage)?;

        let maybe_attrs = get_nft_attributes(&deps.storage, token_id as u64);

        if maybe_attrs.is_none() {
            return Err(StdError::generic_err(format!(
                "Failed to mint - invalid token id: {}",
                token_id
            )));
        }

        let attrs = maybe_attrs.unwrap();

        let public_metadata = Some(create_metadata(&attrs.public_attributes));
        let private_metadata = Some(create_metadata(&attrs.private_attributes));

        mints.push(Mint {
            token_id: Some(token_id.to_string()),
            owner: Some(owner.clone()),
            public_metadata,
            private_metadata,
            serial_number: None,
            // todo: set royalties
            royalty_info: None,
            memo: None,
        });
        tokens_minted.push(token_id.to_string());
    }

    messages.push(batch_mint(mints, None, contract)?);
    Ok(HandleResponse {
        messages,
        log: vec![log("minted", format!("{:?}", tokens_minted))], //plaintext_log <- minted, sender
        data: None,
    })
}

fn create_metadata(attrs: &Attributes) -> Metadata {
    Metadata {
        token_uri: None,
        extension: Some(Extension {
            image: Some(attrs.token_uri.clone()),
            image_data: None,
            external_url: Some(attrs.external_url.clone()),
            description: Some(attrs.description.clone()),
            name: Some(attrs.name.clone()),
            attributes: Some(attrs.custom_traits.clone()),
            background_color: None,
            animation_url: None,
            youtube_url: None,
            media: attrs.media.clone(),
            protected_attributes: None,
        }),
    }
}

pub fn try_mint_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    mint_for: Option<HumanAddr>,
    amount: Option<u8>,
) -> StdResult<HandleResponse> {
    check_admin(deps, &env)?;

    do_mint(deps, mint_for.unwrap_or(env.message.sender), amount)
}

fn try_mint_whitelist<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    mint_for: HumanAddr,
    amount: Option<u8>,
) -> StdResult<HandleResponse> {
    let mut allowed_to_mint = get_whitelist(&deps.storage, &mint_for);

    if allowed_to_mint == 0u8 {
        return Err(StdError::generic_err("Address is not whitelisted"));
    };

    let amount_to_mint = amount.unwrap_or(1);

    if allowed_to_mint < amount_to_mint {
        return Err(StdError::generic_err(format!(
            "Tried to mint more than allowed. Max for this address is: {}",
            allowed_to_mint
        )));
    }

    allowed_to_mint -= amount_to_mint;

    change_allocation(&mut deps.storage, &mint_for, allowed_to_mint)?;

    do_mint(deps, mint_for, amount)
}

pub fn try_mint_with_token<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
    mint_for: HumanAddr,
    mint_amount: u8,
    from: HumanAddr,
) -> StdResult<HandleResponse> {
    let config = config_read(&deps.storage).may_load()?.unwrap();

    let is_whitelist = get_is_whitelist(&config)?;

    // this is a mode that lets us control whether or not everyone can mint or just the platform can
    if config.only_platform {
        if config.platform.is_none() {
            return Err(StdError::generic_err(
                "Only platform can mint but allowed address is undefined",
            ));
        }

        if config.platform.as_ref().unwrap().address != from {
            return Err(StdError::generic_err(
                "Only platform can mint but tried to mint from different address",
            ));
        }
    }

    check_paid_for_mint(
        &config.price,
        &Token::Snip20(Contract {
            address: env.message.sender,
            hash: "".to_string(), // this is just here to reuse the struct
        }),
        amount,
        Some(mint_amount),
        // todo: add whitelist checking
        is_whitelist,
    )?;

    check_cap_amount(&mut deps.storage, &config, mint_amount)?;

    if is_whitelist {
        try_mint_whitelist(deps, mint_for, Some(mint_amount))
    } else {
        do_mint(deps, mint_for, Some(mint_amount))
    }
}

fn check_cap_amount<S: Storage>(storage: &mut S, cfg: &Config, to_mint: u8) -> StdResult<()> {
    let mut cfg = cfg.clone();
    if let Some(cap_amount) = cfg.cap_amount {
        if to_mint as u16 > cap_amount {
            return Err(StdError::generic_err(format!(
                "tried to mint: {}, available: {}",
                to_mint, cap_amount
            )));
        } else {
            cfg.cap_amount = Some(cap_amount - to_mint as u16);
            config(storage).save(&cfg)?;
        }
    }

    Ok(())
}
