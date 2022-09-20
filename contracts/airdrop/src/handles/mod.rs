use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdResult, Storage};

use crate::state::Config;
use crate::types::airdrop::AirdropClaimSubmit;
use crate::types::secret_contract::SecretContract;

mod confirm_airdrop;
use confirm_airdrop::confirm_airdrop;

mod change_config;
mod set_quest_password;

use crate::handles::set_quest_password::handle_set_quest_password;
use change_config::change_config;

mod set_airdrop_vk;
use set_airdrop_vk::set_airdrop_vk;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct NewConfig {
    admin: Option<HumanAddr>,
    confirmer: Option<HumanAddr>,
    platform: Option<SecretContract>,
    token: Option<SecretContract>,
    quest_contract: Option<HumanAddr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    ConfirmAirdrop { airdrops: Vec<AirdropClaimSubmit> },
    ChangeConfig(NewConfig),
    SetQuestPassword { password: String },
    SetAirdropVk(String),
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    use HandleMsg::*;

    let config = Config::load(&deps.storage)?;

    match msg {
        ConfirmAirdrop { airdrops } => confirm_airdrop(deps, env, config, airdrops),
        ChangeConfig(new_config) => change_config(deps, env, config, new_config),
        SetAirdropVk(viewing_key) => set_airdrop_vk(env, config, viewing_key),
        SetQuestPassword { password } => handle_set_quest_password(deps, env, config, password),
    }
}
