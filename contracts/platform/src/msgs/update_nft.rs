use cosmwasm_std::{to_binary, CosmosMsg, HumanAddr, StdResult, WasmMsg};
use secret_toolkit::utils::types::Contract;

use crate::snip721::metadata::Metadata;
use crate::snip721::snip721_handle_msg::{HandleMsg};


#[allow(clippy::too_many_arguments)]
pub fn change_nft_type(
    contract: Contract,
    token_id: String,
    new_type: u8,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract.address,
        callback_code_hash: contract.hash,
        msg: to_binary(&HandleMsg::SetTokenType { token_id, new_type }).unwrap(),
        send: vec![],
    }))
}
