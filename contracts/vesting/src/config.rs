use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{BlockInfo, HumanAddr, ReadonlyStorage, StdError, StdResult, Storage};
use secret_toolkit::storage::{TypedStore, TypedStoreMut};

use crate::types::Contract;

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct Config {
    /// The administrator account
    pub admin: HumanAddr,
    /// the address of this contract, used to validate query permits
    pub contract_address: HumanAddr,
    /// The SNIP-20 token to distribute tokens from
    pub vesting_token: Contract,
    /// The vesting token's viewing key
    pub vesting_token_vk: String,
    /// The details of the last known block
    pub last_block: BlockInfo,
    /// The contract's operational mode
    pub mode: ContractMode,
}

impl Config {
    const STORE_KEY: &'static [u8] = b"config";

    pub fn new(
        admin: HumanAddr,
        contract_address: HumanAddr,
        vesting_token: Contract,
        vesting_token_vk: String,
        last_block: BlockInfo,
    ) -> Self {
        Self {
            admin,
            contract_address,
            vesting_token,
            vesting_token_vk,
            last_block,
            mode: ContractMode::Normal,
        }
    }

    pub fn load<S: ReadonlyStorage>(store: &S) -> StdResult<Self> {
        let config: StdResult<StoredConfig> = TypedStore::attach(store).load(Self::STORE_KEY);
        config.map(|config| config.into())
    }

    pub fn save<S: Storage>(&self, store: &mut S) -> StdResult<()> {
        let conf: StoredConfig = self.clone().into();
        TypedStoreMut::attach(store).store(Self::STORE_KEY, &conf)
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

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ContractMode {
    /// All functionality is enabled
    Normal,
    /// Users can not claim more funds from their vesting accounts
    PausedClaims,
    /// PausedClaims + admin may call EmergencyRedeemAll
    Emergency,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct StoredConfig {
    /// The administrator account
    pub admin: HumanAddr,
    /// the address of this contract, used to validate query permits
    pub contract_address: HumanAddr,
    /// The SNIP-20 token to distribute tokens from
    pub vesting_token: Contract,
    /// The vesting token's viewing key
    pub vesting_token_vk: String,
    /// The details of the last known block
    pub last_block: BlockInfo,
    /// The contract's operational mode
    pub mode: u8,
}

impl From<Config> for StoredConfig {
    fn from(conf: Config) -> Self {
        use ContractMode::*;
        let mode = match conf.mode {
            Normal => 0,
            PausedClaims => 1,
            Emergency => 2,
        };
        Self {
            admin: conf.admin,
            contract_address: conf.contract_address,
            vesting_token: conf.vesting_token,
            vesting_token_vk: conf.vesting_token_vk,
            last_block: conf.last_block,
            mode,
        }
    }
}

impl From<StoredConfig> for Config {
    fn from(conf: StoredConfig) -> Self {
        use ContractMode::*;
        let mode = match conf.mode {
            0 => Normal,
            1 => PausedClaims,
            2 => Emergency,
            _ => panic!("unexpected value in config.mode"),
        };
        Self {
            admin: conf.admin,
            contract_address: conf.contract_address,
            vesting_token: conf.vesting_token,
            vesting_token_vk: conf.vesting_token_vk,
            last_block: conf.last_block,
            mode,
        }
    }
}
