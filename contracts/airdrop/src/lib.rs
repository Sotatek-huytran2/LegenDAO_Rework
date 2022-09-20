mod handles;
mod init;
mod queries;
mod state;
mod types;

pub use handles::handle;
pub use handles::HandleMsg;
pub use init::init;
pub use init::InitMsg;
pub use queries::query;
pub use queries::{AccountInfo, AccountInfoResponse, AirdropClaimResponse, QueryMsg};
pub use types::deposit::{Deposit, LgndReceiveMsg};

pub use crate as contract;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points!(contract);
