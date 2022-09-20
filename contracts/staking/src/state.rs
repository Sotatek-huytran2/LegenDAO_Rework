use cosmwasm_std::{HumanAddr, ReadonlyStorage, StdError, StdResult, Storage, Uint128};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use secret_toolkit::storage::{TypedStore, TypedStoreMut};
use secret_toolkit::utils::types::Contract;
use secret_toolkit_incubator::cashmap::{CashMap, ReadOnlyCashMap};
use serde::{Deserialize, Serialize};

const PREFIX_CONFIG: &[u8] = b"config";
const PREFIX_USER_BALANCES: &[u8] = b"user_balances";
const PREFIX_REWARD_POOL: &[u8] = b"reward_pool";
const PREFIX_REWARD_SCHEDULE: &[u8] = b"reward_schedule";
const PREFIX_SUBSCRIBERS: &[u8] = b"subscribers";
const PREFIX_MULTIPLIER_CONTRACTS: &[u8] = b"multiplier_contracts";
const PREFIX_BOOSTER_ITEMS: &[u8] = b"booster_items";
const PREFIX_USER_INVENTORY: &[u8] = b"user_inventory";
pub const PREFIX_REVOKED_PERMITS: &str = "revoked_permits";

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Config {
    pub admin: HumanAddr,
    pub token: Contract,
    pub platform: Contract,
    pub viewing_key: String,
    pub prng_seed: Vec<u8>,
    pub own_addr: HumanAddr,
    pub max_multiplier: u128,
}

impl Config {
    pub fn load<S: ReadonlyStorage>(storage: &S) -> StdResult<Self> {
        TypedStore::attach(storage).load(PREFIX_CONFIG)
    }

    pub fn save<S: Storage>(&self, storage: &mut S) -> StdResult<()> {
        TypedStoreMut::attach(storage).store(PREFIX_CONFIG, self)
    }
}

/// RewardPool is a struct that keeps track of rewards and lockups
#[derive(Serialize, Deserialize, Default)]
pub struct RewardPool {
    pub residue: u128,
    pub last_reward_block: u64,
    pub total_locked: u128,
    pub total_weight: u128,
    pub acc_reward_per_share: u128,
}

impl RewardPool {
    pub fn load<S: ReadonlyStorage>(storage: &S) -> StdResult<Self> {
        TypedStore::attach(storage).load(PREFIX_REWARD_POOL)
    }

    pub fn save<S: Storage>(&self, storage: &mut S) -> StdResult<()> {
        TypedStoreMut::attach(storage).store(PREFIX_REWARD_POOL, self)
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserBalance {
    pub locked: u128,
    pub debt: u128,
    pub total_multiplier: u128,
    pub weight: u128,
}

impl UserBalance {
    pub fn load<S: ReadonlyStorage>(storage: &S, user: &HumanAddr) -> StdResult<Option<Self>> {
        let user_balances_store = ReadonlyPrefixedStorage::new(PREFIX_USER_BALANCES, storage);
        TypedStore::attach(&user_balances_store).may_load(user.0.as_bytes())
    }

    pub fn save<S: Storage>(&self, storage: &mut S, user: &HumanAddr) -> StdResult<()> {
        let mut user_balances_store = PrefixedStorage::new(PREFIX_USER_BALANCES, storage);
        TypedStoreMut::attach(&mut user_balances_store).store(user.0.as_bytes(), self)
    }
}

impl Default for UserBalance {
    fn default() -> Self {
        UserBalance {
            locked: 0,
            debt: 0,
            total_multiplier: 100_000,
            weight: 0,
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ScheduleUnit {
    end_block: u64,
    reward_per_block: Uint128,
}

impl ScheduleUnit {
    pub fn to_stored(&self) -> StoredScheduleUnit {
        StoredScheduleUnit {
            end_block: self.end_block,
            reward_per_block: self.reward_per_block.u128(),
        }
    }
}

#[cfg(test)]
impl ScheduleUnit {
    pub fn new(end_block: u64, reward_per_block: u128) -> ScheduleUnit {
        ScheduleUnit {
            end_block,
            reward_per_block: Uint128::from(reward_per_block),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct StoredScheduleUnit {
    end_block: u64,
    reward_per_block: u128,
}

impl StoredScheduleUnit {
    pub fn from_stored(&self) -> ScheduleUnit {
        ScheduleUnit {
            end_block: self.end_block,
            reward_per_block: Uint128::from(self.reward_per_block),
        }
    }
}

pub struct InflationSchedule {}

impl InflationSchedule {
    pub fn load<S: ReadonlyStorage>(storage: &S) -> StdResult<Vec<StoredScheduleUnit>> {
        TypedStore::attach(storage).load(PREFIX_REWARD_SCHEDULE)
    }

    pub fn save<S: Storage>(storage: &mut S, schedule: Vec<StoredScheduleUnit>) -> StdResult<()> {
        TypedStoreMut::attach(storage).store(PREFIX_REWARD_SCHEDULE, &schedule)
    }

    pub fn get_inflation<S: ReadonlyStorage>(
        storage: &S,
        from_block: u64,
        to_block: u64,
    ) -> StdResult<u128> {
        if to_block <= from_block {
            return Ok(0);
        }

        let mut from_block = from_block;
        let schedule = Self::load(storage)?;

        let mut amount = 0;
        // Going serially assuming that schedule is not a big vector
        for unit in schedule {
            if from_block < unit.end_block {
                if to_block > unit.end_block {
                    amount += (unit.end_block - from_block) as u128 * unit.reward_per_block;
                    from_block = unit.end_block;
                } else {
                    amount += (to_block - from_block) as u128 * unit.reward_per_block;
                    break; // No need to go further up the schedule
                }
            }
        }

        Ok(amount)
    }
}

pub fn sort_schedule(s: &mut Vec<StoredScheduleUnit>) {
    s.sort_by(|u1, u2| u1.end_block.cmp(&u2.end_block))
}

pub struct Subscribers {}

impl Subscribers {
    pub fn load<S: ReadonlyStorage>(storage: &S) -> StdResult<Vec<Contract>> {
        TypedStore::attach(storage).load(PREFIX_SUBSCRIBERS)
    }

    pub fn save<S: Storage>(storage: &mut S, schedule: Vec<Contract>) -> StdResult<()> {
        TypedStoreMut::attach(storage).store(PREFIX_SUBSCRIBERS, &schedule)
    }
}

pub struct MultiplierContracts {}

impl MultiplierContracts {
    pub fn get<S: ReadonlyStorage>(storage: &S, address: &HumanAddr) -> StdResult<bool> {
        let cash_store: ReadOnlyCashMap<HumanAddr, S> =
            ReadOnlyCashMap::init(PREFIX_MULTIPLIER_CONTRACTS, storage);
        match cash_store.get(address.0.as_bytes()) {
            None => Ok(false),
            Some(_) => Ok(true),
        }
    }

    pub fn add_multiple<S: Storage>(storage: &mut S, addresses: Vec<HumanAddr>) -> StdResult<()> {
        let mut cash_store: CashMap<HumanAddr, S> =
            CashMap::init(PREFIX_MULTIPLIER_CONTRACTS, storage);

        for addr in addresses {
            cash_store.insert(addr.0.as_bytes(), addr.clone())?
        }

        Ok(())
    }

    pub fn remove_multiple<S: Storage>(
        storage: &mut S,
        addresses: Vec<HumanAddr>,
    ) -> StdResult<()> {
        let mut cash_store: CashMap<HumanAddr, S> =
            CashMap::init(PREFIX_MULTIPLIER_CONTRACTS, storage);

        for addr in addresses {
            cash_store.remove(addr.0.as_bytes())?;
        }

        Ok(())
    }

    pub fn require_multiplier_contract<S: ReadonlyStorage>(
        storage: &S,
        address: &HumanAddr,
    ) -> StdResult<()> {
        match Self::get(storage, address)? {
            true => Ok(()),
            false => Err(StdError::generic_err(format!(
                "address {} is not allowed to set multipliers",
                address
            ))),
        }
    }

    pub fn get_page<S: ReadonlyStorage>(
        storage: &S,
        page_number: Option<u32>,
        page_size: u32,
    ) -> StdResult<Vec<HumanAddr>> {
        let cash_store =
            ReadOnlyCashMap::<HumanAddr, S>::init(PREFIX_MULTIPLIER_CONTRACTS, storage);

        let start_page = page_number.unwrap_or(0u32);
        cash_store.paging(start_page, page_size)
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct BoosterItemInInventory {
    pub multiplier: u32,
    pub contract: HumanAddr,
    pub id: String,
}

impl BoosterItemInInventory {
    fn remove<S: Storage>(
        storage: &mut S,
        owner: &HumanAddr,
        contract: &HumanAddr,
        token_id: &str,
    ) -> StdResult<()> {
        let user_namespace = [PREFIX_USER_INVENTORY, owner.0.as_bytes()].concat();
        let mut user_cash_store: CashMap<BoosterItemInInventory, S> =
            CashMap::init(&user_namespace, storage);

        let item_key = [contract.0.as_bytes(), token_id.as_bytes()].concat();
        user_cash_store.remove(&item_key)
    }

    fn save<S: Storage>(
        storage: &mut S,
        owner: &HumanAddr,
        contract: &HumanAddr,
        token_id: &str,
        multiplier: u32,
    ) -> StdResult<()> {
        let user_namespace = [PREFIX_USER_INVENTORY, owner.0.as_bytes()].concat();
        let mut user_cash_store: CashMap<BoosterItemInInventory, S> =
            CashMap::init(&user_namespace, storage);

        let item_key = [contract.0.as_bytes(), token_id.as_bytes()].concat();
        user_cash_store.insert(
            &item_key,
            BoosterItemInInventory {
                multiplier,
                contract: contract.clone(),
                id: token_id.to_string(),
            },
        )
    }

    pub fn get_inventory_page<S: Storage>(
        storage: &S,
        owner: &HumanAddr,
        start_page: u32,
        size: u32,
    ) -> StdResult<Vec<BoosterItemInInventory>> {
        let user_namespace = [PREFIX_USER_INVENTORY, owner.0.as_bytes()].concat();
        let user_cash_store: ReadOnlyCashMap<BoosterItemInInventory, S> =
            ReadOnlyCashMap::init(&user_namespace, storage);

        user_cash_store.paging(start_page, size)
    }
}

#[derive(Serialize, Deserialize)]
pub struct BoosterItem {
    pub owner: HumanAddr,
    pub multiplier: u32,
}

impl BoosterItem {
    pub fn get<S: ReadonlyStorage>(
        storage: &S,
        contract: &HumanAddr,
        token_id: &str,
    ) -> StdResult<Option<Self>> {
        let booster_items_from_contract = ReadonlyPrefixedStorage::multilevel(
            &[PREFIX_BOOSTER_ITEMS, contract.0.as_bytes()],
            storage,
        );
        TypedStore::attach(&booster_items_from_contract).may_load(token_id.as_bytes())
    }

    pub fn remove<S: Storage>(
        storage: &mut S,
        contract: &HumanAddr,
        token_id: &str,
        owner: &HumanAddr,
    ) -> StdResult<()> {
        let mut booster_items_from_contract =
            PrefixedStorage::multilevel(&[PREFIX_BOOSTER_ITEMS, contract.0.as_bytes()], storage);
        TypedStoreMut::<Self, _>::attach(&mut booster_items_from_contract)
            .remove(token_id.as_bytes());

        // remove from user's inventory
        // assumes the owner is correct, was verified on get_multiplier
        BoosterItemInInventory::remove(storage, owner, contract, token_id)
    }

    pub fn save<S: Storage>(
        &self,
        storage: &mut S,
        contract: &HumanAddr,
        token_id: &str,
    ) -> StdResult<()> {
        let mut booster_items_from_contract =
            PrefixedStorage::multilevel(&[PREFIX_BOOSTER_ITEMS, contract.0.as_bytes()], storage);
        TypedStoreMut::attach(&mut booster_items_from_contract).store(token_id.as_bytes(), self)?;

        // save to owner's inventory
        BoosterItemInInventory::save(storage, &self.owner, contract, token_id, self.multiplier)
    }

    pub fn is_locked<S: Storage>(
        storage: &S,
        contract: &HumanAddr,
        token_id: &str,
    ) -> StdResult<bool> {
        match Self::get(storage, contract, token_id)? {
            None => Ok(false),
            Some(_) => Ok(true),
        }
    }

    pub fn get_multiplier<S: Storage>(
        storage: &S,
        contract: &HumanAddr,
        token_id: &str,
        expected_owner: &HumanAddr,
    ) -> StdResult<Option<u32>> {
        match Self::get(storage, contract, token_id)? {
            None => Ok(None),
            Some(item) => {
                if &item.owner == expected_owner {
                    Ok(Some(item.multiplier))
                } else {
                    Err(StdError::generic_err(format!(
                        "Item {} was not locked by {}",
                        token_id, expected_owner
                    )))
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum Features {
    Deposit,
    Withdraw,
    EmergencyWithdraw,
    EmergencyWithdrawSkipPlatform,
}
