use cosmwasm_std::{HumanAddr, ReadonlyStorage, StdResult, Storage};
use cosmwasm_storage::{
    singleton, singleton_read, PrefixedStorage, ReadonlyPrefixedStorage, ReadonlySingleton,
    Singleton,
};
use schemars::JsonSchema;
use secret_toolkit::storage::{AppendStore, AppendStoreMut};
use secret_toolkit::utils::types::Contract;
use serde::{Deserialize, Serialize};

use crate::msg::MintPrice;
use crate::types::minting_level::MintingLevel;

pub static RANDOM_NUMBERS: &[u8] = b"r";
pub static CONFIG_KEY: &[u8] = b"config";
pub static RNG_CONFIG_KEY: &[u8] = b"rng_config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub nft_count: u16,
    pub cap_amount: Option<u16>,
    pub owner: HumanAddr,
    pub nft_contract: Contract,
    pub max_batch_mint: u8,
    pub is_revealed: bool,
    pub minting_enabled: MintingLevel,
    pub price: Vec<MintPrice>,
    pub platform: Option<Contract>,
    /// If this contract can only be accessed via the platform
    pub only_platform: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OptionalConfig {
    pub nft_count: Option<u16>,
    pub owner: Option<HumanAddr>,
    pub nft_contract: Option<Contract>,
    pub max_batch_mint: Option<u8>,
    pub is_revealed: Option<bool>,
    pub minting_enabled: Option<MintingLevel>,
    pub price: Option<Vec<MintPrice>>,
    pub platform: Option<Option<Contract>>,
    /// If this contract can only be accessed via the platform
    pub only_platform: Option<bool>,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG_KEY)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RngConfig {
    pub entropy: Vec<u8>,
    pub seed: Vec<u8>,
    pub counter: u64,
}

pub fn rng_config<S: Storage>(storage: &mut S) -> Singleton<S, RngConfig> {
    singleton(storage, RNG_CONFIG_KEY)
}

pub fn rng_config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, RngConfig> {
    singleton_read(storage, RNG_CONFIG_KEY)
}

pub fn u64_to_bytes(number: &u64) -> [u8; 8] {
    number.to_be_bytes()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NftInventoryConfig {
    pub num_of_items: u32,
}

pub fn build_random_numbers<S: Storage>(store: &mut S, amount: u16) -> StdResult<()> {
    let mut store = PrefixedStorage::new(RANDOM_NUMBERS, store);
    let mut store = AppendStoreMut::attach_or_create(&mut store)?;

    for i in 1..=amount {
        store.push(&i)?;
    }

    Ok(())
}

pub fn pop_number_from_storage<S: Storage>(store: &mut S, pos: u32) -> StdResult<u16> {
    let mut store = PrefixedStorage::new(RANDOM_NUMBERS, store);
    let mut append_store = AppendStoreMut::attach_or_create(&mut store)?;

    match append_store.len() {
        0 => Ok(0),
        1 => append_store.pop(),
        _ if append_store.len() - 1 == pos => append_store.pop(),
        _ => {
            let result = append_store.get_at(pos)?;
            let last = append_store.pop()?;
            append_store.set_at(pos, &last)?;

            Ok(result)
        }
    }
}

pub fn numbers_remaining<S: ReadonlyStorage>(store: &S) -> StdResult<u32> {
    let store = ReadonlyPrefixedStorage::new(RANDOM_NUMBERS, store);
    let store: AppendStore<u16, ReadonlyPrefixedStorage<S>> =
        AppendStore::attach(&store).unwrap()?;

    Ok(store.len())
}
