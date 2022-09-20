use crate::contract::do_claim;
use crate::state::{Balances, Config};
use cosmwasm_std::{CosmosMsg, Env, HumanAddr, ReadonlyStorage, StdResult, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{DequeStore, DequeStoreMut};
use serde::{Deserialize, Serialize};

const PREFIX_CLAIMS: &[u8] = b"claims";

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Claim {
    account: HumanAddr,
    end_ts: u64,
}

pub struct AutoClaims {}

impl AutoClaims {
    pub fn len<S: ReadonlyStorage>(storage: &S) -> StdResult<u32> {
        let queue_store = ReadonlyPrefixedStorage::new(PREFIX_CLAIMS, storage);
        let claims = match DequeStore::<Claim, ReadonlyPrefixedStorage<S>>::attach(&queue_store) {
            None => return Ok(0),
            Some(c) => c?,
        };

        Ok(claims.len())
    }

    pub fn peek_next_claim<S: ReadonlyStorage>(storage: &S, env: &Env) -> StdResult<Option<Claim>> {
        let queue_store = ReadonlyPrefixedStorage::new(PREFIX_CLAIMS, storage);
        let claims = match DequeStore::<Claim, ReadonlyPrefixedStorage<S>>::attach(&queue_store) {
            None => return Ok(None),
            Some(c) => c?,
        };

        match claims.iter().next() {
            None => Ok(None),
            Some(c) => {
                let c: Claim = c?;
                if c.end_ts > env.block.time {
                    return Ok(None);
                }

                Ok(Some(c))
            }
        }
    }

    pub fn pop_next_claim<S: Storage>(storage: &mut S, env: &Env) -> StdResult<Option<Claim>> {
        if Self::peek_next_claim(storage, env)?.is_some() {
            let mut queue_store = PrefixedStorage::new(PREFIX_CLAIMS, storage);
            let mut claims = DequeStoreMut::attach_or_create(&mut queue_store)?;

            return match claims.pop_front() {
                Ok(c) => Ok(Some(c)),
                Err(_) => Ok(None),
            };
        }

        Ok(None)
    }

    fn push_claim<S: Storage>(storage: &mut S, claim: Claim) -> StdResult<()> {
        let mut queue_store = PrefixedStorage::new(PREFIX_CLAIMS, storage);
        let mut claims = DequeStoreMut::attach_or_create(&mut queue_store)?;

        claims.push_back(&claim)
    }

    pub fn new_unbonding<S: Storage>(
        storage: &mut S,
        env: &Env,
        config: &Config,
        user: HumanAddr,
        user_balances: &mut Balances,
        amount: u128,
    ) -> StdResult<()> {
        let (unbonding, is_new) = user_balances
            .pending_redeem
            .create_or_add_to_existing_unbonding_record(env, config, amount)?;

        if is_new {
            Self::push_claim(
                storage,
                Claim {
                    account: user,
                    end_ts: unbonding.end_ts,
                },
            )?;
        }

        Ok(())
    }

    pub fn claim_next_pending_msg<S: Storage>(
        storage: &mut S,
        env: &Env,
    ) -> StdResult<Option<Vec<CosmosMsg>>> {
        let claim = match Self::pop_next_claim(storage, env)? {
            None => return Ok(None),
            Some(c) => c,
        };

        do_claim(storage, env, &claim.account)
    }
}
