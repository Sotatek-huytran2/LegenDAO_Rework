use serde::Serialize;

use cosmwasm_std::{HumanAddr, Uint128};

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LgndReceiveMsg {
    /// Deposit funds in platform
    #[allow(unused)] // Copied from the platform contract but not used in practice
    Deposit {
        /// The account for which the funds will be deposited
        to: HumanAddr,
    },
    BatchDeposit(Vec<Deposit>),
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Deposit {
    pub to: HumanAddr,
    pub amount: Uint128,
}

impl Deposit {
    pub fn new(to: HumanAddr, amount: Uint128) -> Self {
        Self { to, amount }
    }
}
