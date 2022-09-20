use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::secret_contract::SecretContract;
use cosmwasm_std::{HumanAddr, ReadonlyStorage, StdError, StdResult, Storage};

use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{TypedStore, TypedStoreMut};

const QUEST_CONTRACT: &[u8] = b"quest_contract";
const QUEST_STATUS: &[u8] = b"quest_status";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: HumanAddr,
    pub token: SecretContract,
    pub platform: SecretContract,
    pub airdrop_contract: Option<SecretContract>,
    pub password: Option<String>,
}

impl Config {
    const STORE_KEY: &'static [u8] = b"config";

    pub fn new(admin: HumanAddr, token: SecretContract, platform: SecretContract) -> Self {
        Self {
            admin,
            token,
            platform,
            airdrop_contract: None,
            password: None,
        }
    }

    pub fn load<S: ReadonlyStorage>(store: &S) -> StdResult<Self> {
        TypedStore::attach(store).load(Self::STORE_KEY)
    }

    pub fn save<S: Storage>(&self, store: &mut S) -> StdResult<()> {
        TypedStoreMut::attach(store).store(Self::STORE_KEY, self)
    }

    pub fn assert_admin(&self, address: &HumanAddr) -> StdResult<()> {
        if address != &self.admin {
            return Err(StdError::generic_err(format!(
                "Address {} is not allowed to perform this operation",
                address
            )));
        }
        Ok(())
    }
}

pub fn add_quest_contract<S: Storage>(
    store: &mut S,
    address: &HumanAddr,
    quest: u8,
) -> StdResult<()> {
    let mut store = PrefixedStorage::new(QUEST_CONTRACT, store);
    let mut typed_store = TypedStoreMut::attach(&mut store);
    typed_store.store(address_to_bytes(address), &quest.to_be_bytes())
}

pub fn get_quest_contract<S: ReadonlyStorage>(store: &S, address: &HumanAddr) -> u8 {
    let store = ReadonlyPrefixedStorage::new(QUEST_CONTRACT, store);
    let typed_store = TypedStore::attach(&store);
    let result = typed_store.may_load(address_to_bytes(address));

    result.unwrap_or(None).unwrap_or(0)
}

pub fn add_quest_weight<S: Storage>(store: &mut S, quest: u8, weight: u8) -> StdResult<()> {
    let mut typed_store = TypedStoreMut::attach(store);
    typed_store.store(&quest.to_be_bytes(), &weight.to_be_bytes())
}

pub fn get_quest_weight<S: ReadonlyStorage>(store: &S, quest: u8) -> u8 {
    let typed_store = TypedStore::attach(store);
    let result = typed_store.may_load(&quest.to_be_bytes());

    result.unwrap_or(None).unwrap_or(0)
}

pub fn set_quest_status<S: Storage>(
    store: &mut S,
    address: &HumanAddr,
    quest: u8,
) -> StdResult<()> {
    let mut store = PrefixedStorage::new(QUEST_STATUS, store);
    let mut typed_store = TypedStoreMut::attach(&mut store);
    typed_store.store(address_to_bytes(address), &quest.to_be_bytes())
}

pub fn get_quest_status<S: ReadonlyStorage>(store: &S, address: &HumanAddr) -> u8 {
    let store = ReadonlyPrefixedStorage::new(QUEST_STATUS, store);
    let typed_store = TypedStore::attach(&store);
    let result = typed_store.may_load(address_to_bytes(address));

    result.unwrap_or(None).unwrap_or(0)
}

pub fn address_to_bytes(address: &HumanAddr) -> &[u8] {
    address.0.as_bytes()
}
