use cosmwasm_std::{to_binary, Empty, CosmosMsg, HumanAddr, StdResult, WasmMsg, WasmQuery, QueryRequest};
use secret_toolkit::utils::types::Contract;

// use crate::msg::QueryMsg;
use crate::snip721::snip721_handle_msg::{QueryMsg};


#[allow(clippy::too_many_arguments)]
pub fn get_token_type(
    contract: Contract,
    token_id: String,
) -> StdResult<QueryRequest<Empty>> {
    Ok(QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract.address,
        callback_code_hash: contract.hash,
        msg: to_binary(&QueryMsg::TokenType { token_id }).unwrap(),
    }))
}

// #[allow(clippy::too_many_arguments)]
// pub fn get_token_type(
//     contract: Contract,
//     token_id: String,
// ) -> StdResult<QueryRequest<WasmQuery>> {
//     Ok(QueryRequest::Wasm(WasmQuery::Smart {
//         contract_addr: contract.address,
//         callback_code_hash: contract.hash,
//         msg: to_binary(TokenType {
//             token_id: token_id,
//         }).unwrap(),
//     }))
// }

// #[allow(clippy::too_many_arguments)]
// pub fn token_type(
//     contract: Contract,
//     token_type: u8,
// ) -> StdResult<QueryRequest<WasmQuery>> {
//     Ok(QueryRequest::Wasm(WasmQuery::Smart {
//         contract_addr: contract.address,
//         callback_code_hash: contract.hash,
//         msg: to_binary(&QueryMsg::TokenTypeRespone { token_type }).unwrap(),
//     }))
// }

