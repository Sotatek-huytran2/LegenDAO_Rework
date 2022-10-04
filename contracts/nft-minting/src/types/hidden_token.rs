use schemars::JsonSchema;

use cosmwasm_std::Storage;
use cosmwasm_storage::{singleton, Singleton};
use serde::{Deserialize, Serialize};

pub static HIDDEN_TOKEN_CONFIG: &[u8] = b"htoken";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HiddenToken {
    pub token_uri: String,
}

pub fn hidden_token<S: Storage>(storage: &mut S) -> Singleton<S, HiddenToken> {
    singleton(storage, HIDDEN_TOKEN_CONFIG)
}

// pub fn hidden_token_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, HiddenToken> {
//     singleton_read(storage, HIDDEN_TOKEN_CONFIG)
// }
