use std::cmp::min;
use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{BlockInfo, HumanAddr, ReadonlyStorage, StdError, StdResult, Storage, Uint128};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{TypedStore, TypedStoreMut};

// This is strictly used in the contract interface
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct NewSchedule {
    pub start_time: u64,
    pub allocation: Uint128,
    pub rate: Uint128,
    pub releases: Vec<u16>,
}

impl From<NewSchedule> for StoredSchedule {
    fn from(sched: NewSchedule) -> Self {
        Self {
            start_time: sched.start_time,
            allocation: sched.allocation.u128(),
            rate: sched.rate.u128(),
            releases: sched.releases,
            claimed: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StoredSchedule {
    start_time: u64,
    allocation: u128,
    rate: u128,
    releases: Vec<u16>,
    claimed: u128,
}

impl From<StoredSchedule> for Schedule {
    fn from(sched: StoredSchedule) -> Self {
        Self {
            start_time: sched.start_time,
            allocation: Uint128(sched.allocation),
            rate: Uint128(sched.rate),
            releases: sched.releases,
            claimed: Uint128(sched.claimed),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Schedule {
    start_time: u64,
    allocation: Uint128,
    rate: Uint128,
    releases: Vec<u16>,
    claimed: Uint128,
}

impl StoredSchedule {
    /// Return the total amount of funds that have been released up to the time at the block
    pub fn released_at(&self, block: &BlockInfo) -> u128 {
        let current_time = block.time;
        let mut start_time = self.start_time;

        if current_time < start_time {
            return 0;
        }

        let mut elapsed_periods = 1_u64;
        for release in self.releases.iter() {
            let interval_seconds = *release as u64 * 24 * 60 * 60;
            if (current_time - start_time) < interval_seconds {
                break;
            }
            elapsed_periods += 1;
            start_time += interval_seconds;
        }

        let theoretical_allocation = elapsed_periods as u128 * self.rate;

        min(theoretical_allocation, self.allocation)
    }

    /// Return the amount of funds still available to claim.
    pub fn available_at(&self, block: &BlockInfo) -> u128 {
        // No need for checked arithmetic because we know we didn't claim more than is released
        self.released_at(block) - self.claimed
    }

    /// Increase the amount of claimed funds, while checking if the user has enough.
    ///
    /// The amount to claim is echoed back for API consistency with `claim_all`
    pub fn claim(&mut self, to_claim: u128, block: &BlockInfo) -> StdResult<u128> {
        let available = self.available_at(block);
        if to_claim > available {
            return Err(StdError::generic_err(format!(
                "Not enough funds are available to withdraw yet. {} > {}",
                to_claim, available
            )));
        }

        self.claimed += to_claim;

        Ok(to_claim)
    }

    /// Set the claimed funds to equal the released funds, and return what the difference was between them.
    pub fn claim_all(&mut self, block: &BlockInfo) -> u128 {
        let released = self.released_at(block);
        let to_claim = released - self.claimed;
        self.claimed = released;

        to_claim
    }
}

pub struct Vesting;

impl Vesting {
    const STORE_KEY: &'static [u8] = b"vesting";
    const TOTAL_ALLOCATION_KEY: &'static [u8] = b"vesting-total-allocation";

    pub fn get_total_allocation<S: ReadonlyStorage>(store: &S) -> StdResult<u128> {
        TypedStore::attach(store)
            .may_load(Self::TOTAL_ALLOCATION_KEY)
            .map(|maybe| maybe.unwrap_or(0))
    }

    pub fn get_schedule<S: ReadonlyStorage>(
        store: &S,
        address: &HumanAddr,
    ) -> StdResult<Option<StoredSchedule>> {
        let store = ReadonlyPrefixedStorage::new(Self::STORE_KEY, store);
        let store = TypedStore::attach(&store);
        store.may_load(address.as_str().as_bytes())
    }

    /// Update the state of the vesting schedules.
    ///
    /// This method lets you describe how the vesting schedule should be changed,
    /// and then performs all the changes AFTER the closure runs, and before returning
    /// from this method
    pub fn update<S, F, T>(store: &mut S, updater: F) -> StdResult<T>
    where
        S: Storage,
        F: FnOnce(&mut VestingContext<S>) -> StdResult<T>,
    {
        // Describe how to update the allocation
        let mut context = VestingContext::new(store);
        let value = updater(&mut context)?;

        let mut total_allocation: u128 = TypedStore::attach(store)
            .may_load(Self::TOTAL_ALLOCATION_KEY)?
            .unwrap_or(0);
        let initial_allocation = total_allocation;

        // destructure `context` because we need to release the shared lifetime on the storage.
        let mut read_schedules = context.read_schedules;
        let schedules = context.schedules;

        // Update the schedules for each of the accounts
        let mut pstore = PrefixedStorage::new(Self::STORE_KEY, store);
        let mut sched_store = TypedStoreMut::attach(&mut pstore);
        for (address, schedule) in schedules.into_iter() {
            let addr_bytes = address.as_str().as_bytes();

            // Check if we already loaded the schedule for an account
            let prev_schedule = match read_schedules.remove(&address) {
                Some(Some(prev_sched)) => Some(prev_sched),
                // Otherwise, load it from storage
                _ => sched_store.may_load(addr_bytes)?,
            };

            if let Some(prev_schedule) = &prev_schedule {
                // Should always be safe without a bounds check
                total_allocation -= prev_schedule.allocation;
            }

            if let Some(schedule) = schedule {
                // Save the schedule and update the allocation
                total_allocation = total_allocation
                    .checked_add(schedule.allocation)
                    .ok_or_else(|| {
                        StdError::generic_err(format!(
                            "Trying to allocate vesting funds for {} exceeded u128::MAX",
                            address
                        ))
                    })?;
                sched_store.store(addr_bytes, &schedule)?;
            } else {
                sched_store.remove(addr_bytes);
            }
        }

        // Save the new allocation
        if initial_allocation != total_allocation {
            TypedStoreMut::attach(store).store(Self::TOTAL_ALLOCATION_KEY, &total_allocation)?;
        }

        Ok(value)
    }
}

pub struct VestingContext<'store, S: ReadonlyStorage> {
    // fields NOT pub
    /// Used when the updater wants to read vesting schedule
    store: &'store S,
    /// The new states to write to the db
    schedules: HashMap<HumanAddr, Option<StoredSchedule>>,
    /// A cache. If the user reads schedules, we won't go to the storage again for them later.
    read_schedules: HashMap<HumanAddr, Option<StoredSchedule>>,
}

impl<'store, S: ReadonlyStorage> VestingContext<'store, S> {
    // NOT pub
    fn new(store: &'store S) -> Self {
        Self {
            store,
            schedules: HashMap::new(),
            read_schedules: HashMap::new(),
        }
    }

    /// Get read access to the storage
    #[allow(dead_code)]
    pub fn get_store(&self) -> &S {
        self.store
    }

    /// Convenience for fetching the total allocation
    #[allow(dead_code)]
    pub fn get_total_allocation(&self) -> StdResult<u128> {
        Vesting::get_total_allocation(self.store)
    }

    /// Read a vesting schedule from the cache.
    ///
    /// This is the preferred method of access, as it reduces the amount of reads later when saving.
    pub fn get_schedule(&mut self, address: &HumanAddr) -> StdResult<Option<StoredSchedule>> {
        // Check the caches first
        if let Some(schedule) = self.schedules.get(address) {
            return Ok(schedule.clone());
        }
        if let Some(schedule) = self.read_schedules.get(address) {
            return Ok(schedule.clone());
        }

        // Otherwise access the storage as usual
        let schedule = Vesting::get_schedule(self.store, address);

        // If storage access was successful, save the schedule in the cache
        if let Ok(schedule) = &schedule {
            self.read_schedules
                .insert(address.clone(), schedule.clone());
        }
        schedule
    }

    /// Mark the address's schedule to be saved in the storage
    pub fn set_schedule(&mut self, address: HumanAddr, schedule: StoredSchedule) {
        self.schedules.insert(address, Some(schedule));
    }

    /// Mark the address's schedule to be removed from the storage
    pub fn remove_schedule(&mut self, address: HumanAddr) {
        // None marks that the account should be removed
        self.schedules.insert(address, None);
    }
}
