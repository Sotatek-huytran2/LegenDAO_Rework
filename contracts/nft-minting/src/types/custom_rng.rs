use crate::state::pop_number_from_storage;
use cosmwasm_std::{StdError, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use secret_toolkit::crypto::{sha_256, Prng};
use serde::{Deserialize, Serialize};

pub static CUSTOM_RNG: &[u8] = b"r";

#[derive(Serialize, Deserialize, Clone)]
pub struct NftRng {
    rng_seed: Vec<u8>,
    counter: u32,
    remaining: u16,
}

impl NftRng {
    pub fn new(initial_nfts: u16) -> Self {
        NftRng {
            rng_seed: vec![],
            counter: 0,
            remaining: initial_nfts,
        }
    }

    #[allow(dead_code)]
    pub fn random_seed(&mut self) -> [u8; 32] {
        let mut base_rng = Prng::new(self.rng_seed.as_slice(), vec![].as_slice());
        base_rng.rand_bytes()
    }

    #[allow(dead_code)]
    pub fn random_number(&mut self, to: u16, entropy: &[u8]) -> u16 {
        let mut base_rng = Prng::new(self.rng_seed.as_slice(), &sha_256(entropy));
        let bytes = base_rng.rand_bytes();

        let be_bytes: [u8; 2] = [bytes[0], bytes[1]];

        u16::from_be_bytes(be_bytes) % to
    }

    pub fn append_randomness(&mut self, rand: &[u8]) {
        self.rng_seed.extend_from_slice(rand)
    }

    pub fn next<S: Storage>(&mut self, store: &mut S) -> StdResult<u16> {
        if self.remaining == 0 {
            return Err(StdError::generic_err("No more items"));
        }

        let mut base_rng = Prng::new(self.rng_seed.as_slice(), vec![].as_slice());
        base_rng.set_word_pos(self.counter);

        self.counter += 1;

        let bytes = base_rng.rand_bytes();

        let be_bytes: [u8; 2] = [bytes[0], bytes[1]];

        let random_number = u16::from_be_bytes(be_bytes) % self.remaining;

        self.remaining -= 1;

        // should be 0 to number of items remaining
        pop_number_from_storage(store, random_number as u32)

        // let id: u16 = self.counter as u16;   
        // Ok(id)
    }

    //fn select_one_of(&mut self) {}

    pub fn save<S: Storage>(&self, storage: &mut S) -> StdResult<()> {
        let mut sg: Singleton<S, Self> = singleton(storage, CUSTOM_RNG);

        sg.save(self)
    }

    pub fn load<S: Storage>(storage: &S) -> StdResult<Self> {
        let sg: ReadonlySingleton<S, Self> = singleton_read(storage, CUSTOM_RNG);

        Ok(sg.may_load()?.unwrap())
    }

    pub fn remaining(&self) -> u16 {
        self.remaining
    }
}