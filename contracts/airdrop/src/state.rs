use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::secret_contract::SecretContract;
use cosmwasm_std::{HumanAddr, ReadonlyStorage, StdError, StdResult, Storage};

use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{TypedStore, TypedStoreMut};

pub static AIRDROP_CLAIMS: &[u8] = b"airdrop_claims";
pub static ACCOUNT_TOTALS: &[u8] = b"account_totals";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: HumanAddr,
    pub confirmer: HumanAddr,
    pub platform: SecretContract,
    pub token: SecretContract,
    pub quest_password: Option<String>,
    pub quest_contract: Option<HumanAddr>,
}

impl Config {
    const STORE_KEY: &'static [u8] = b"config";

    pub fn new(
        admin: HumanAddr,
        confirmer: HumanAddr,
        platform: SecretContract,
        token: SecretContract,
        quest_contract: Option<HumanAddr>,
    ) -> Self {
        Self {
            admin,
            confirmer,
            platform,
            token,
            quest_password: None,
            quest_contract,
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

    pub fn assert_quest_contract(&self, address: &HumanAddr) -> StdResult<()> {
        if let Some(config_quest_contract) = &self.quest_contract {
            if config_quest_contract == address {
                return Ok(());
            }
        }

        Err(StdError::generic_err(format!(
            "Address {} is not allowed to perform this operation",
            address
        )))
    }

    pub fn assert_password(&self, password: &str) -> StdResult<()> {
        if let Some(config_quest_password) = &self.quest_password {
            if config_quest_password == password {
                return Ok(());
            }
        }

        Err(StdError::generic_err(
            "Password does not match or is not set",
        ))
    }

    pub fn assert_confirmer(&self, address: &HumanAddr) -> StdResult<()> {
        if address != &self.confirmer {
            return Err(StdError::generic_err(format!(
                "Address {} is not allowed to perform this operation",
                address
            )));
        }
        Ok(())
    }
}

pub fn set_claimed<S: Storage>(store: &mut S, address: &str) -> StdResult<()> {
    let mut store = PrefixedStorage::new(AIRDROP_CLAIMS, store);
    let mut store = TypedStoreMut::attach(&mut store);

    store.store(address.as_bytes(), &true)?;

    Ok(())
}

pub fn is_claimed<S: ReadonlyStorage>(store: &S, address: &str) -> bool {
    let store = ReadonlyPrefixedStorage::new(AIRDROP_CLAIMS, store);
    let append_store = TypedStore::attach(&store);

    append_store.load(address.as_bytes()).unwrap_or_default()
}

pub fn address_to_bytes(address: &HumanAddr) -> &[u8] {
    address.0.as_bytes()
}

pub fn add_allocation_for_address<S: Storage>(
    store: &mut S,
    address: &HumanAddr,
    amount: u128,
) -> StdResult<()> {
    let mut store = PrefixedStorage::new(ACCOUNT_TOTALS, store);
    let mut typed_store = TypedStoreMut::attach(&mut store);
    typed_store.store(address_to_bytes(address), &amount)
}

pub fn get_allocation_for_account<S: ReadonlyStorage>(store: &S, address: &HumanAddr) -> u128 {
    let store = ReadonlyPrefixedStorage::new(ACCOUNT_TOTALS, store);
    let typed_store = TypedStore::attach(&store);
    let result = typed_store.may_load(address_to_bytes(address));

    result.unwrap_or(None).unwrap_or(0)
}

// pub fn change_allocation<S: Storage>(
//     store: &mut S,
//     address: &HumanAddr,
//     amount: u8,
// ) -> StdResult<()> {
//     let mut typed_store = TypedStoreMut::attach(store);
//
//     return if amount == 0 {
//         typed_store.remove(address_to_bytes(address));
//         Ok(())
//     } else {
//         typed_store.store(address_to_bytes(address), &amount.to_be_bytes())
//     };
// }
