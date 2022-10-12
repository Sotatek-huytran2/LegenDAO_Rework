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

#[allow(clippy::too_many_arguments)]
pub fn change_nft_metadata(
    contract: Contract,
    token_id: String,
    public_metadata: Option<Metadata>,
    private_metadata: Option<Metadata>,
    padding: Option<String>
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract.address,
        callback_code_hash: contract.hash,
        msg: to_binary(&HandleMsg::SetMetadata { token_id, public_metadata, private_metadata, padding }).unwrap(),
        send: vec![],
    }))
}

#[allow(clippy::too_many_arguments)]
pub fn burn_loot_box(
    contract: Contract,
    token_id: String,
    memo: Option<String>,
    padding: Option<String>
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract.address,
        callback_code_hash: contract.hash,
        msg: to_binary(&HandleMsg::BurnNft { token_id, memo, padding }).unwrap(),
        send: vec![],
    }))
}
