use cosmwasm_std::{to_binary, CosmosMsg, WasmMsg};
use secret_toolkit::utils::types::Contract;

use crate::snip721::snip721_handle_msg::HandleMsg;

pub fn enable_reveal_msg(contract: Contract) -> CosmosMsg {
    CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract.address,
        callback_code_hash: contract.hash,
        msg: to_binary(&HandleMsg::EnableReveal {}).unwrap(),
        send: vec![],
    })
}
