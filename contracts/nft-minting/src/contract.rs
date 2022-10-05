use cosmwasm_std::{
    from_binary, log, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HandleResult,
    HumanAddr, InitResponse, Querier, StdResult, Storage, Uint128,
};
use secret_toolkit::snip20;
use secret_toolkit::utils::types::Contract;

use crate::handles::add_whitelist::add_whitelist;
use crate::handles::change_settings::change_settings;
use crate::handles::enable_reveal::try_enable_reveal;
use crate::handles::mint::{try_mint_admin, try_mint_native, try_mint_with_token};
use crate::handles::remove_whitelist::remove_whitelist;
use crate::handles::set_attributes::try_set_attributes;
use crate::handles::set_minting_level::set_minting_level;
use crate::handles::set_placeholder::set_placeholder;
use crate::handles::withdraw::withdraw_funds;
use crate::msg::{HandleMsg, InitMsg, PlatformApi, QueryMsg, ReceiveMsg, Token};
use crate::queries::is_whitelisted::query_is_whitelisted;
use crate::queries::minting_level::query_minting_level;
use crate::queries::remaining::query_remaining;
use crate::state::{build_random_numbers, config, Config};
use crate::types::custom_rng::NftRng;
use crate::types::minting_level::MintingLevel;

const MAX_MINT_AT_ONCE: u8 = 100;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let mut rng = NftRng::new(msg.nft_count);
    rng.append_randomness(msg.random_seed.as_slice());
    rng.save(&mut deps.storage)?;

    build_random_numbers(&mut deps.storage, msg.nft_count)?;

    let mut messages: Vec<CosmosMsg> = msg
        .price
        .iter()
        .filter_map(|price| match &price.token {
            Token::Snip20(Contract { address, hash }) => Some(
                snip20::register_receive_msg(
                    env.contract_code_hash.clone(),
                    None,
                    256,
                    hash.clone(),
                    address.clone(),
                )
                .unwrap(),
            ),
            _ => None,
        })
        .collect();

    let vk_messages: Vec<CosmosMsg> = msg
        .price
        .iter()
        .filter_map(|price| match &price.token {
            Token::Snip20(Contract { address, hash }) => Some(
                snip20::set_viewing_key_msg(
                    "balanceVK".parse().unwrap(),
                    None,
                    256,
                    hash.clone(),
                    address.clone(),
                )
                .unwrap(),
            ),
            _ => None,
        })
        .collect();

    messages.extend(vk_messages);

    let state = Config {
        nft_count: msg.nft_count,
        base_uri: msg.base_uri,
        cap_amount: None,
        owner: env.message.sender,
        nft_contract: msg.nft_contract,
        max_batch_mint: MAX_MINT_AT_ONCE,
        is_revealed: false,
        minting_enabled: MintingLevel::AdminOnly,
        price: msg.price,
        platform: msg.platform,
        only_platform: msg.only_platform.unwrap_or(false),
    };

    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse {
        messages,
        log: vec![log("status", "success")],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::AddWhitelist { addresses } => add_whitelist(deps, env, addresses),
        HandleMsg::RemoveWhitelist { addresses } => remove_whitelist(deps, env, addresses),
        HandleMsg::SetPlaceHolder { token_uri } => set_placeholder(deps, env, token_uri),
        HandleMsg::MintAdmin { amount, amount_loot_box_to_mint, amount_item_to_mint, mint_for } => try_mint_admin(deps, env, mint_for, amount, amount_loot_box_to_mint, amount_item_to_mint),
        HandleMsg::Mint { amount, amount_loot_box_to_mint, amount_item_to_mint } => try_mint_native(deps, env, amount, amount_loot_box_to_mint, amount_item_to_mint),
        HandleMsg::EnableReveal {} => try_enable_reveal(deps, env),
        HandleMsg::Receive { amount, msg, from } => {
            try_receive_from_platform(deps, env, amount, msg, from)
        }
        HandleMsg::ChangingMintingState {
            mint_state,
            cap_amount,
        } => set_minting_level(deps, env, mint_state, cap_amount),
        HandleMsg::WithdrawFunds {
            dest,
            amount,
            snip20_msg,
            token,
        } => withdraw_funds(deps, env, dest, token, amount, snip20_msg),
        HandleMsg::SetAttributes { tokens } => try_set_attributes(deps, env, tokens),
        HandleMsg::Cleanup {} => Ok(HandleResponse::default()),
        HandleMsg::ChangeConfig { settings } => change_settings(deps, env, settings),
    }
}

fn try_receive_from_platform<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
    msg_external: Option<Binary>,
    from_external: HumanAddr,
) -> HandleResult {
    let unwrapped_msg = msg_external.unwrap_or_default();

    // let msg_platform_api: StdResult<PlatformApi> = from_binary(&unwrapped_msg);

    if let Ok(msg_platform_api) = from_binary::<PlatformApi>(&unwrapped_msg) {
        match msg_platform_api {
            PlatformApi::ReceiveFromPlatform { msg, .. } => {
                receive(deps, env, amount, msg, from_external)
            }
        }
    } else {
        receive(deps, env, amount, unwrapped_msg, from_external)
    }
}

fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
    msg: Binary,
    from: HumanAddr,
) -> HandleResult {
    let msg: ReceiveMsg = from_binary(&msg)?;

    match msg {
        ReceiveMsg::Mint {
            mint_for,
            amount_avatar_to_mint,
            amount_loot_box_to_mint,
            amount_item_to_mint,
        } => try_mint_with_token(deps, env, amount, mint_for, amount_avatar_to_mint, amount_loot_box_to_mint, amount_item_to_mint, from),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Remaining {} => to_binary(&query_remaining(deps)?),
        QueryMsg::MintingLevel {} => to_binary(&query_minting_level(deps)?),
        QueryMsg::IsWhitelisted { address } => to_binary(&query_is_whitelisted(deps, address)?),
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::msg::{MintPrice, RemainingResponse};
//     use crate::state::pop_number_from_storage;
//     use crate::types::token_attributes::{Attributes, CoinAttributes, InputTokenAttributes};
//     use cosmwasm_std::testing::{mock_dependencies, mock_env};
//     use cosmwasm_std::{coins, from_binary, Uint128};
//     use secret_toolkit::utils::types::Contract;

//     use super::*;

//     #[test]
//     fn proper_initialization() {
//         let mut deps = mock_dependencies(20, &[]);

//         let msg = InitMsg {
//             nft_count: 200,
//             nft_contract: Contract {
//                 address: Default::default(),
//                 hash: "".to_string(),
//             },
//             random_seed: Binary(vec![0u8, 0u8, 0u8, 0u8]),
//             price: vec![MintPrice {
//                 token: Token::Native("uscrt".to_string()),
//                 price: Uint128::zero(),
//                 whitelist_price: Uint128::zero(),
//             }],
//             platform: None,
//             only_platform: None,
//         };
//         let env = mock_env("creator", &coins(1000, "uscrt"));

//         // we can just call .unwrap() to assert this was a success
//         let res = init(&mut deps, env, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         // should increase counter by 1
//         let res = query(&deps, QueryMsg::Remaining {}).unwrap();
//         let value: RemainingResponse = from_binary(&res).unwrap();
//         assert_eq!(200, value.remaining);
//     }

//     #[test]
//     fn mint_single() {
//         let mut deps = mock_dependencies(20, &[]);

//         let msg = InitMsg {
//             nft_count: 200,
//             nft_contract: Contract {
//                 address: Default::default(),
//                 hash: "".to_string(),
//             },
//             random_seed: Binary(vec![0u8, 0u8, 0u8, 0u8]),
//             price: vec![MintPrice {
//                 token: Token::Native("uscrt".to_string()),
//                 price: Uint128::from(1u64),
//                 whitelist_price: Uint128::from(1u64),
//             }],
//             platform: None,
//             only_platform: None,
//         };
//         let env = mock_env("creator", &coins(1000, "uscrt"));

//         // we can just call .unwrap() to assert this was a success
//         let res = init(&mut deps, env, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         let res = _set_attributues(&mut deps, 200);
//         assert_eq!(0, res.messages.len());

//         let res = _enable_mint(&mut deps);
//         assert_eq!(0, res.messages.len());

//         let res = _mint_request(&mut deps);
//         assert_eq!(res.log.len(), 1);
//         assert_eq!(&res.log[0].value, "[\"30\"]");

//         let res = query(&deps, QueryMsg::Remaining {}).unwrap();
//         let value: RemainingResponse = from_binary(&res).unwrap();
//         assert_eq!(199, value.remaining);
//     }

//     #[test]
//     fn mint_multiple() {
//         let mut deps = mock_dependencies(20, &[]);

//         let msg = InitMsg {
//             nft_count: 200,
//             nft_contract: Contract {
//                 address: Default::default(),
//                 hash: "".to_string(),
//             },
//             random_seed: Binary(vec![0u8, 0u8, 0u8, 0u8]),
//             price: vec![MintPrice {
//                 token: Token::Native("uscrt".to_string()),
//                 price: Uint128::from(1u64),
//                 whitelist_price: Uint128::from(1u64),
//             }],
//             platform: None,
//             only_platform: None,
//         };
//         let env = mock_env("creator", &coins(1000, "uscrt"));

//         // we can just call .unwrap() to assert this was a success
//         let res = init(&mut deps, env, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         let res = _enable_mint(&mut deps);
//         assert_eq!(0, res.messages.len());

//         let res = _set_attributues(&mut deps, 200);
//         assert_eq!(0, res.messages.len());

//         let res = _mint_request(&mut deps);
//         assert_eq!(res.log.len(), 1);
//         assert_eq!(&res.log[0].value, "[\"30\"]");

//         let res = _mint_request(&mut deps);
//         assert_eq!(res.log.len(), 1);
//         assert_eq!(&res.log[0].value, "[\"38\"]");

//         let res = _mint_request(&mut deps);
//         assert_eq!(res.log.len(), 1);
//         assert_eq!(&res.log[0].value, "[\"120\"]");
//     }

//     #[test]
//     fn mint_a_lot() {
//         let mut deps = mock_dependencies(20, &[]);

//         let num_of_tokens = 10_000;

//         let msg = InitMsg {
//             nft_count: num_of_tokens as u16,
//             nft_contract: Contract {
//                 address: Default::default(),
//                 hash: "".to_string(),
//             },
//             random_seed: Binary(vec![0u8, 0u8, 0u8, 0u8]),
//             price: vec![MintPrice {
//                 token: Token::Native("uscrt".to_string()),
//                 price: Uint128::from(1u64),
//                 whitelist_price: Uint128::from(1u64),
//             }],
//             platform: None,
//             only_platform: None,
//         };
//         let env = mock_env("creator", &coins(100000, "uscrt"));

//         // we can just call .unwrap() to assert this was a success
//         let res = init(&mut deps, env, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         let res = _enable_mint(&mut deps);
//         assert_eq!(0, res.messages.len());

//         let res = _set_attributues(&mut deps, num_of_tokens);
//         assert_eq!(0, res.messages.len());

//         for _i in 0..num_of_tokens {
//             let res = _mint_request(&mut deps);
//             assert_eq!(res.log.len(), 1);
//         }
//     }

//     #[test]
//     fn test_mint_last_position() {
//         let mut deps = mock_dependencies(20, &[]);

//         let msg = InitMsg {
//             nft_count: 200,
//             nft_contract: Contract {
//                 address: Default::default(),
//                 hash: "".to_string(),
//             },
//             random_seed: Binary(vec![0u8, 0u8, 0u8, 0u8]),
//             price: vec![MintPrice {
//                 token: Token::Native("uscrt".to_string()),
//                 price: Uint128::from(1u64),
//                 whitelist_price: Uint128::from(1u64),
//             }],
//             platform: None,
//             only_platform: None,
//         };
//         let env = mock_env("creator", &coins(1000, "uscrt"));

//         // we can just call .unwrap() to assert this was a success
//         let res = init(&mut deps, env, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         let result = pop_number_from_storage(&mut deps.storage, 200);
//         assert!(result.is_err(), "{:?}", result);

//         let result = pop_number_from_storage(&mut deps.storage, 199);
//         assert!(result.is_ok(), "{:?}", result);
//     }

//     fn _mint_request<S: Storage, A: Api, Q: Querier>(
//         mut deps: &mut Extern<S, A, Q>,
//     ) -> HandleResponse {
//         let env = mock_env("creator", &coins(1, "uscrt"));
//         let msg = HandleMsg::Mint { amount: None };
//         let res = handle(&mut deps, env, msg).unwrap();
//         res
//     }

//     fn _enable_mint<S: Storage, A: Api, Q: Querier>(
//         mut deps: &mut Extern<S, A, Q>,
//     ) -> HandleResponse {
//         let env = mock_env("creator", &coins(1, "uscrt"));
//         let msg = HandleMsg::ChangingMintingState {
//             mint_state: MintingLevel::Public,
//             cap_amount: None,
//         };
//         let res = handle(&mut deps, env, msg).unwrap();
//         res
//     }

//     fn _set_attributues<S: Storage, A: Api, Q: Querier>(
//         mut deps: &mut Extern<S, A, Q>,
//         num_of_tokens: u16,
//     ) -> HandleResponse {
//         let env = mock_env("creator", &coins(1, "uscrt"));

//         let mut attribute_vec = vec![];
//         for i in 1..=num_of_tokens {
//             attribute_vec.push(InputTokenAttributes {
//                 token_id: i.to_string(),
//                 attributes: CoinAttributes {
//                     public_attributes: Attributes {
//                         custom_traits: vec![],
//                         //rarity: Rarity::Legendary,
//                         description: "".to_string(),
//                         name: "".to_string(),
//                         external_url: "".to_string(),
//                         media: None,
//                         token_uri: "".to_string(),
//                     },
//                     private_attributes: Attributes {
//                         custom_traits: vec![],
//                         //rarity: Rarity::Legendary,
//                         description: "".to_string(),
//                         name: "".to_string(),
//                         external_url: "".to_string(),
//                         media: Some(vec![crate::snip721::extension::MediaFile {
//                             file_type: None,
//                             extension: None,
//                             authentication: None,
//                             url: "this is a test".to_string()
//                         }]),
//                         token_uri: "".to_string(),
//                     },
//                 },
//             })
//         }

//         let msg = HandleMsg::SetAttributes {
//             tokens: attribute_vec,
//         };
//         let res = handle(&mut deps, env, msg).unwrap();
//         res
//     }
// }
