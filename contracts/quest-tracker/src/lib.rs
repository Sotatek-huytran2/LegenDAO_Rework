mod handles;
mod init;
mod queries;
mod state;
mod types;

pub use handles::HandleMsg;
pub use init::InitMsg;
pub use queries::QueryMsg;

use handles::handle;
use init::init;
use queries::query;

use crate as contract;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points!(contract);
