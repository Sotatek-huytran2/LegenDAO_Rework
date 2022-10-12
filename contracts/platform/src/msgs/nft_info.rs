use cosmwasm_std::{to_binary, CosmosMsg, HumanAddr, StdResult, WasmMsg, WasmQuery, QueryRequest};
use secret_toolkit::utils::types::Contract;

use crate::snip721::metadata::Metadata;
use crate::snip721::snip721_handle_msg::{QueryMsg};


#[allow(clippy::too_many_arguments)]
pub fn token_type(
    contract: Contract,
    token_id: String,
) -> StdResult<QueryRequest<WasmQuery>> {
    Ok(QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract.address,
        callback_code_hash: contract.hash,
        msg: to_binary(&QueryMsg::TokenType { token_id }).unwrap(),
    }))
}
