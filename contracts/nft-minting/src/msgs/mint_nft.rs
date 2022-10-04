use cosmwasm_std::{to_binary, CosmosMsg, HumanAddr, StdResult, WasmMsg};
use secret_toolkit::utils::types::Contract;

use crate::snip721::metadata::Metadata;
use crate::snip721::snip721_handle_msg::{HandleMsg, Mint};

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
pub fn mint_nft_msg(
    token_id: Option<String>,
    owner: Option<HumanAddr>,
    public_metadata: Option<Metadata>,
    private_metadata: Option<Metadata>,
    memo: Option<String>,
    padding: Option<String>,
    _block_size: usize,
    contract: Contract,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract.address,
        callback_code_hash: contract.hash,
        msg: to_binary(&HandleMsg::MintNft {
            token_id,
            owner,
            public_metadata,
            private_metadata,
            serial_number: None,
            royalty_info: None,
            memo,
            padding,
        })
        .unwrap(),
        send: vec![],
    }))
}

#[allow(clippy::too_many_arguments)]
pub fn batch_mint(
    mints: Vec<Mint>,
    padding: Option<String>,
    contract: Contract,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract.address,
        callback_code_hash: contract.hash,
        msg: to_binary(&HandleMsg::BatchMintNft { mints, padding }).unwrap(),
        send: vec![],
    }))
}
