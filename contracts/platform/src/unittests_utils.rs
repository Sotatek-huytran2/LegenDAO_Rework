// #![cfg(test)]
// use std::any::Any;

// use cosmwasm_std::testing::{
//     mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
// };
// use cosmwasm_std::{
//     from_binary, BankMsg, Binary, BlockInfo, Coin, ContractInfo, CosmosMsg, Env, Extern,
//     HandleResponse, HumanAddr, MessageInfo, QueryResponse, StdError, StdResult, Uint128,
// };
// use secret_toolkit::serialization::Base64JsonOf;
// use secret_toolkit::snip20;
// use secret_toolkit::utils::feature_toggle::{FeatureToggleHandleMsg, FeatureToggleQueryMsg};
// use secret_toolkit::utils::types::Contract;

// use crate::constants::RESPONSE_BLOCK_SIZE;
// use crate::contract::{handle, init, query};
// use crate::msg::HandleMsg::Redeem;
// use crate::msg::{
//     HandleAnswer, HandleMsg, InitMsg, QueryAnswer, QueryMsg, ReceiveMsg, ResponseTotalBalances,
// };
// use crate::state::{Features, TotalBalances};

// pub fn init_helper(
//     receiving_contracts: Option<Vec<HumanAddr>>,
//     token_address: Option<HumanAddr>,
// ) -> StdResult<Extern<MockStorage, MockApi, MockQuerier>> {
//     let mut deps = mock_dependencies(20, &[]);
//     let token_address = token_address.unwrap_or(HumanAddr::from("token"));

//     init(
//         &mut deps,
//         mock_env_with_time(HumanAddr::from("admin"), 1),
//         InitMsg {
//             token: Contract {
//                 address: token_address,
//                 hash: "whatever".to_string(),
//             },
//             token_native_denom: "ibc/WHATEVER".to_string(),
//             unbonding_period: None,
//             receiving_contracts,
//             viewing_key: "".to_string(),
//         },
//     )?;

//     Ok(deps)
// }

// pub fn mock_env_with_time<U: Into<HumanAddr>>(sender: U, time: u64) -> Env {
//     Env {
//         block: BlockInfo {
//             height: 12_345,
//             time,
//             chain_id: "cosmos-testnet-14002".to_string(),
//         },
//         message: MessageInfo {
//             sender: sender.into(),
//             sent_funds: vec![],
//         },
//         contract: ContractInfo {
//             address: HumanAddr::from(MOCK_CONTRACT_ADDR),
//         },
//         contract_key: Some("".to_string()),
//         contract_code_hash: "".to_string(),
//     }
// }

// pub fn extract_messages(hr: HandleResponse) -> Vec<CosmosMsg> {
//     let HandleResponse { messages, .. } = hr;
//     messages
// }

// pub fn extract_answer(hr: HandleResponse) -> StdResult<HandleAnswer> {
//     let HandleResponse { data, .. } = hr;
//     from_binary(&data.unwrap())
// }

// pub fn extract_query_answer(qr: QueryResponse) -> StdResult<QueryAnswer> {
//     from_binary(&qr)
// }

// pub fn create_redeem_message(amount: u128) -> StdResult<CosmosMsg> {
//     snip20::redeem_msg(
//         Uint128::from(amount),
//         Some("ibc/WHATEVER".to_string()),
//         None,
//         RESPONSE_BLOCK_SIZE,
//         "whatever".to_string(),
//         HumanAddr::from("token"),
//     )
// }

// pub fn create_bank_send_message(from: &str, to: &str, amount: u128) -> CosmosMsg {
//     CosmosMsg::Bank(BankMsg::Send {
//         from_address: HumanAddr::from(from),
//         to_address: HumanAddr::from(to),
//         amount: vec![Coin {
//             denom: "ibc/WHATEVER".to_string(),
//             amount: Uint128::from(amount),
//         }],
//     })
// }

// pub fn create_send_message(to: &str, amount: u128, msg: Option<Binary>) -> StdResult<CosmosMsg> {
//     snip20::send_msg(
//         HumanAddr::from(to),
//         Uint128::from(amount),
//         msg,
//         None,
//         None,
//         RESPONSE_BLOCK_SIZE,
//         "whatever".to_string(),
//         HumanAddr::from("token"),
//     )
// }

// pub fn extract_generic_error_msg<T: Any>(error: StdResult<T>) -> String {
//     match error {
//         Ok(_) => {
//             panic!("Handle Response is not an error")
//         }
//         Err(err) => match err {
//             cosmwasm_std::StdError::GenericErr { msg, .. } => msg,
//             _ => panic!("Error is not generic"),
//         },
//     }
// }

// pub fn do_deposit(
//     deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
//     to: &str,
//     amount: u128,
//     time: u64,
// ) -> StdResult<HandleResponse> {
//     let result = handle(
//         deps,
//         mock_env_with_time(HumanAddr::from("token"), time),
//         HandleMsg::Receive {
//             sender: HumanAddr::from(to),
//             from: HumanAddr::from(to),
//             amount: Uint128::from(amount),
//             msg: Base64JsonOf::from(ReceiveMsg::Deposit {
//                 to: HumanAddr::from(to),
//             }),
//         },
//     )?;

//     Ok(result)
// }

// pub fn pause_feature(
//     deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
//     feature: Features,
//     time: u64,
//     pauser: &str,
// ) -> StdResult<String> {
//     let result = handle(
//         deps,
//         mock_env_with_time(HumanAddr::from(pauser), time),
//         HandleMsg::Features(FeatureToggleHandleMsg::Pause {
//             features: vec![feature],
//         }),
//     )?;

//     let data = result.data.unwrap().0;
//     String::from_utf8(data.clone()).map_err(|_| StdError::invalid_utf8(""))
// }

// pub fn add_pauser(
//     deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
//     pauser: &str,
//     time: u64,
//     admin: Option<&str>,
// ) -> StdResult<String> {
//     let result = handle(
//         deps,
//         mock_env_with_time(HumanAddr::from(admin.unwrap_or("admin")), time),
//         HandleMsg::Features(FeatureToggleHandleMsg::SetPauser {
//             address: HumanAddr::from(pauser),
//         }),
//     )?;

//     let data = result.data.unwrap().0;
//     String::from_utf8(data.clone()).map_err(|_| StdError::invalid_utf8(""))
// }

// pub fn do_redeem(
//     deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
//     from: &str,
//     amount: u128,
//     time: u64,
// ) -> StdResult<Vec<CosmosMsg>> {
//     let result = handle(
//         deps,
//         mock_env_with_time(HumanAddr::from(from), time),
//         Redeem {
//             amount: Some(Uint128::from(amount)),
//         },
//     )?;
//     let result_messages = extract_messages(result);

//     Ok(result_messages)
// }

// pub fn do_claim(
//     deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
//     from: &str,
//     time: u64,
// ) -> StdResult<Vec<CosmosMsg>> {
//     let result = handle(
//         deps,
//         mock_env_with_time(HumanAddr::from(from), time),
//         HandleMsg::ClaimRedeemed {},
//     )?;
//     let result_messages = extract_messages(result);

//     Ok(result_messages)
// }

// pub fn claim_json(account: &str, end_ts: u64) -> String {
//     format!("{{\"account\":\"{}\",\"end_ts\":{}}}", account, end_ts)
// }

// pub fn do_send_from_platform(
//     deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
//     from: &str,
//     contract_addr: HumanAddr,
//     amount: Option<Uint128>,
// ) -> StdResult<Vec<CosmosMsg>> {
//     let result = handle(
//         deps,
//         mock_env(HumanAddr::from(from), &[]),
//         HandleMsg::SendFromPlatform {
//             contract_addr,
//             amount,
//             memo: None,
//             msg: Default::default(),
//         },
//     )?;
//     let result_messages = extract_messages(result);

//     Ok(result_messages)
// }

// pub fn do_set_viewing_key(
//     deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
//     from: &str,
//     key: &str,
// ) -> StdResult<HandleAnswer> {
//     let result = handle(
//         deps,
//         mock_env(HumanAddr::from(from), &[]),
//         HandleMsg::SetViewingKey {
//             key: key.to_string(),
//             padding: Some("pad".to_string()),
//         },
//     )?;

//     extract_answer(result)
// }

// pub fn expect_platform_total_balance(
//     deps: &Extern<MockStorage, MockApi, MockQuerier>,
//     expected_staking: u128,
//     expected_unbonding: u128,
// ) -> StdResult<()> {
//     let result = query(&deps, QueryMsg::TotalBalances {})?;
//     let parsed_result = extract_query_answer(result)?;

//     if let QueryAnswer::TotalBalances(total_balances) = parsed_result {
//         assert_eq!(
//             total_balances,
//             ResponseTotalBalances::from(TotalBalances {
//                 staked: expected_staking,
//                 unbonding: expected_unbonding,
//             })
//         );
//     } else {
//         panic!("unexpected query answer variant");
//     }

//     Ok(())
// }

// pub fn query_is_pauser(
//     deps: &Extern<MockStorage, MockApi, MockQuerier>,
//     address: &str,
// ) -> StdResult<String> {
//     let result = query(
//         &deps,
//         QueryMsg::Features(FeatureToggleQueryMsg::IsPauser {
//             address: HumanAddr::from(address),
//         }),
//     )?;

//     let data = result.0;
//     String::from_utf8(data.clone()).map_err(|_| StdError::invalid_utf8(""))
// }

// pub fn query_feature_status(
//     deps: &Extern<MockStorage, MockApi, MockQuerier>,
//     feature: Features,
// ) -> StdResult<String> {
//     let result = query(
//         &deps,
//         QueryMsg::Features(FeatureToggleQueryMsg::Status {
//             features: vec![feature],
//         }),
//     )?;

//     let data = result.0;
//     String::from_utf8(data.clone()).map_err(|_| StdError::invalid_utf8(""))
// }
