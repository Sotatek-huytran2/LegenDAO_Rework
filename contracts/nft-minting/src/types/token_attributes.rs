use crate::snip721::extension::MediaFile;
use cosmwasm_std::{ReadonlyStorage, StdResult, Storage};
use schemars::JsonSchema;
use secret_toolkit::storage::{TypedStore, TypedStoreMut};
use serde::{Deserialize, Serialize};

use crate::snip721::snip721_trait::Trait;

use crate::state::u64_to_bytes;
//use crate::types::rarity::Rarity;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Attributes {
    pub custom_traits: Vec<Trait>,
    pub description: String,
    pub name: String,
    pub external_url: String,
    pub media: Option<Vec<MediaFile>>,
    //pub rarity: Rarity,
    pub token_uri: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CoinAttributes {
    pub public_attributes: Attributes,
    pub private_attributes: Attributes,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InputTokenAttributes {
    pub token_id: String,
    pub attributes: CoinAttributes,
}

pub fn set_nft_attributes<S: Storage>(
    store: &mut S,
    token_id: u64,
    attrs: &CoinAttributes,
) -> StdResult<()> {
    let mut typed_store = TypedStoreMut::attach(store);
    typed_store.store(&u64_to_bytes(&token_id), attrs)
}

pub fn get_nft_attributes<S: ReadonlyStorage>(store: &S, token_id: u64) -> Option<CoinAttributes> {
    let typed_store = TypedStore::attach(store);
    let result = typed_store.may_load(&u64_to_bytes(&token_id));

    if result.is_err() {
        return None;
    }

    result.unwrap()
}
