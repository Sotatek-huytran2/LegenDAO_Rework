use cosmwasm_std::{Env, HumanAddr, ReadonlyStorage, StdError, StdResult, Storage, WasmQuery, QueryRequest, Binary};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use secret_toolkit::storage::{TypedStore, TypedStoreMut};
use secret_toolkit::utils::types::Contract;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::snip721::snip721_handle_msg::TokenTypeRespone;

pub const SECONDS_IN_DAY: u64 = 60 * 60 * 24;

// Keys

const PREFIX_CONFIG: &[u8] = b"config";
const PREFIX_BALANCES: &[u8] = b"balances";
const PREFIX_RECEIVING_CONTRACTS: &[u8] = b"receiving_contracts";
const PREFIX_TOTAL_BALANCE: &[u8] = b"total_balance";

// Stored Types

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Config {
    pub admin: HumanAddr,
    pub distribute_address: HumanAddr,
    pub token: Contract,
    pub legen_dao_nft: Contract,
    pub native_token_denom: String,
    pub unbonding_period: u64,
    pub self_contract_addr: HumanAddr,
    pub signer_address: Binary,
}

impl Config {
    pub fn load<S: ReadonlyStorage>(storage: &S) -> StdResult<Option<Self>> {
        TypedStore::attach(storage).may_load(PREFIX_CONFIG)
    }

    pub fn save<S: Storage>(&self, storage: &mut S) -> StdResult<()> {
        TypedStoreMut::attach(storage).store(PREFIX_CONFIG, self)
    }
}

impl Config {
    pub fn get_unchecked<S: ReadonlyStorage>(storage: &S) -> StdResult<Self> {
        Ok(Self::load(storage)?
            .unwrap_or_else(|| panic!("can't load config, storage is probably corrupted")))
    }

    pub fn require_admin(&self, env: &Env) -> StdResult<()> {
        if self.admin != env.message.sender {
            return Err(StdError::unauthorized());
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Balances {
    /// Staked amount, not including unbonding (or claimable) funds
    pub staked: u128,
    /// Withdraw requests
    pub pending_redeem: RedeemInfo,
}

impl Balances {
    pub fn load<S: ReadonlyStorage>(storage: &S, key: &HumanAddr) -> StdResult<Option<Self>> {
        let balances_store = ReadonlyPrefixedStorage::new(PREFIX_BALANCES, storage);
        TypedStore::attach(&balances_store).may_load(key.0.as_bytes())
    }

    pub fn save<S: Storage>(&self, storage: &mut S, key: &HumanAddr) -> StdResult<()> {
        let mut balances_store = PrefixedStorage::new(PREFIX_BALANCES, storage);
        TypedStoreMut::attach(&mut balances_store).store(key.0.as_bytes(), self)
    }
}

// Types

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct UnbondingRecord {
    /// Unbonding period ending timestamp (in seconds)
    pub end_ts: u64,
    /// Amount unbonding
    pub amount: u128,
}

impl UnbondingRecord {
    pub fn is_over(&self, env: &Env) -> bool {
        self.end_ts < env.block.time
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct RedeemInfo {
    /// Unbonding withdraws
    pub unbondings: Vec<UnbondingRecord>, // Sorted by start time, bulked by day
    /// Claimable withdraws (i.e. finished unbonding period)
    pub claimable: u128,
}

impl RedeemInfo {
    pub fn refresh(&mut self, env: &Env) {
        // Relies on `self.unbondings` to be sorted
        while let Some(u) = self.unbondings.last() {
            if u.is_over(env) {
                self.claimable += u.amount;
                self.unbondings.pop();
            } else {
                break;
            }
        }

        self.unbondings.shrink_to_fit();
    }

    pub fn find_unbonding_index_by_time(&mut self, end_ts: u64) -> usize {
        if self.unbondings.is_empty() {
            return 0;
        }

        // Relies on `self.unbondings` to be sorted by `end_ts`.
        for (i, u) in self.unbondings.iter().enumerate() {
            if u.end_ts <= end_ts {
                return i;
            }
        }

        self.unbondings.len() - 1
    }

    /// Returns `true` if created a new record or `false` if didn't
    pub fn create_or_add_to_existing_unbonding_record(
        &mut self,
        env: &Env,
        config: &Config,
        amount: u128,
    ) -> StdResult<(UnbondingRecord, bool)> {
        let bulk_end_ts =
            env.block.time - (env.block.time % SECONDS_IN_DAY) + config.unbonding_period;
        let new_record = UnbondingRecord {
            end_ts: bulk_end_ts,
            amount,
        };
        let index = self.find_unbonding_index_by_time(bulk_end_ts);

        // The `self.unbondings` vector is sorted: |[idx=0] larger ts|------>|[idx=n] smaller ts|
        // i.e. normally, a new unbonding will be inserted at the beginning
        if let Some(unbonding) = self.unbondings.get_mut(index) {
            match unbonding.end_ts.cmp(&bulk_end_ts) {
                Ordering::Less => {
                    self.unbondings.splice(index..index, [new_record.clone()]);
                }
                Ordering::Equal => {
                    unbonding.amount += amount;
                    return Ok((unbonding.clone(), false));
                }
                Ordering::Greater => {
                    self.unbondings
                        .resize_with(self.unbondings.len() + 1, || new_record.clone());
                }
            }
        } else {
            self.unbondings.push(new_record.clone());
        }

        Ok((new_record, true))
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum Features {
    Redeem,
    Claim,
    SendFromPlatform,
    Deposit,
}

pub struct ReceivingContracts {}

impl ReceivingContracts {
    pub fn get<S: ReadonlyStorage>(storage: &S, address: &HumanAddr) -> StdResult<bool> {
        let store = ReadonlyPrefixedStorage::new(PREFIX_RECEIVING_CONTRACTS, storage);
        match TypedStore::attach(&store).may_load(address.0.as_bytes())? {
            None => Ok(false),
            Some(s) => Ok(s),
        }
    }

    pub fn set_multiple<S: Storage>(storage: &mut S, addresses: Vec<HumanAddr>) -> StdResult<()> {
        let mut store = PrefixedStorage::new(PREFIX_RECEIVING_CONTRACTS, storage);
        let mut typed_store = TypedStoreMut::attach(&mut store);

        for addr in addresses {
            typed_store.store(addr.0.as_bytes(), &true)?
        }

        Ok(())
    }

    pub fn remove_multiple<S: Storage>(storage: &mut S, addresses: Vec<HumanAddr>) {
        let mut store = PrefixedStorage::new(PREFIX_RECEIVING_CONTRACTS, storage);
        let mut typed_store = TypedStoreMut::<bool, PrefixedStorage<S>>::attach(&mut store);

        for addr in addresses {
            typed_store.remove(addr.0.as_bytes())
        }
    }

    pub fn require_receiving<S: ReadonlyStorage>(
        storage: &S,
        address: &HumanAddr,
    ) -> StdResult<()> {
        match Self::get(storage, address)? {
            true => Ok(()),
            false => Err(StdError::generic_err(format!(
                "address {} is not a receiving contract, sending tokens from platform is not allowed",
                address
            ))),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct TotalBalances {
    pub staked: u128,
    pub unbonding: u128,
}

impl TotalBalances {
    pub fn load<S: ReadonlyStorage>(storage: &S) -> StdResult<Option<Self>> {
        TypedStore::attach(storage).may_load(PREFIX_TOTAL_BALANCE)
    }

    pub fn save<S: Storage>(&self, storage: &mut S) -> StdResult<()> {
        TypedStoreMut::attach(storage).store(PREFIX_TOTAL_BALANCE, self)
    }
}

impl TotalBalances {
    pub fn handle_balance_change<S: Storage>(
        storage: &mut S,
        change: BalanceChange,
        amount: u128,
    ) -> StdResult<()> {
        let mut balances = Self::load(storage)?.unwrap_or_default();
        match change {
            BalanceChange::Redeem => {
                balances.staked -= amount;
                balances.unbonding += amount;
            }
            BalanceChange::Claim => balances.unbonding -= amount,
            BalanceChange::Deposit => balances.staked += amount,
            BalanceChange::Send => balances.staked -= amount,
        }
        balances.save(storage)
    }

    pub fn to_query_result(&self) -> Self {
        TotalBalances {
            staked: leave_n_most_significant_digits(self.staked, 3),
            unbonding: leave_n_most_significant_digits(self.unbonding, 3),
        }
    }
}

fn leave_n_most_significant_digits(num: u128, digits: u32) -> u128 {
    let base = 10_u128;

    // Check if `num` is big enough to meaningfully obfuscate
    if num == 0 || base.pow(digits) >= num {
        return num;
    }

    let mut min_oom = 0;
    let mut check_oom;
    let mut max_oom = 38;

    // Find `num`'s decimal order of magnitude
    while max_oom - min_oom > 1 {
        check_oom = min_oom + (max_oom - min_oom) / 2;

        if num / base.pow(check_oom) == 0 {
            max_oom = check_oom;
        } else {
            min_oom = check_oom;
        }
    }

    // if the min and max are 1 number apart, we need to check which is the right one.
    check_oom = match num / base.pow(min_oom) > base {
        true => max_oom,
        false => min_oom,
    };

    let filter = base.pow(1 + check_oom - digits);

    num / filter * filter
}

pub enum BalanceChange {
    Redeem,
    Claim,
    Deposit,
    Send,
}
