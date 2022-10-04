use cosmwasm_std::{HumanAddr, ReadonlyStorage, StdResult, Storage};
use secret_toolkit::storage::{TypedStore, TypedStoreMut};

pub fn address_to_bytes(address: &HumanAddr) -> &[u8] {
    address.0.as_bytes()
}

pub fn add_to_whitelist<S: Storage>(
    store: &mut S,
    address: &HumanAddr,
    amount: u8,
) -> StdResult<()> {
    let mut typed_store = TypedStoreMut::attach(store);
    typed_store.store(address_to_bytes(address), &amount.to_be_bytes())
}

pub fn get_whitelist<S: ReadonlyStorage>(store: &S, address: &HumanAddr) -> u8 {
    let typed_store = TypedStore::attach(store);
    let result = typed_store.may_load(address_to_bytes(address));

    result.unwrap_or(None).unwrap_or(0)
}

pub fn change_allocation<S: Storage>(
    store: &mut S,
    address: &HumanAddr,
    amount: u8,
) -> StdResult<()> {
    let mut typed_store = TypedStoreMut::attach(store);

    return if amount == 0 {
        typed_store.remove(address_to_bytes(address));
        Ok(())
    } else {
        typed_store.store(address_to_bytes(address), &amount.to_be_bytes())
    };
}
