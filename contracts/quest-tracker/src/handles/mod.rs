use schemars::JsonSchema;
use serde::Deserialize;

use cosmwasm_std::{Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdResult, Storage};

use crate::state::Config;
use crate::types::secret_contract::SecretContract;

mod complete_quest;
use complete_quest::complete_quest;

mod change_config;
mod change_quest_weight;
mod register_airdrop_contract;
mod register_quest_contract;
mod remove_quest_contract;

use crate::handles::register_airdrop_contract::register_airdrop_contract;
use change_config::change_config;
use change_quest_weight::change_quest_weight;
use register_quest_contract::register_quest_contract;
use remove_quest_contract::remove_quest_contract;

#[derive(Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct NewConfig {
    admin: Option<HumanAddr>,
    // platform: Option<SecretContract>,
    token: Option<SecretContract>,
    airdrop_contract: Option<SecretContract>,
    platform: Option<SecretContract>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    CompleteQuest { address: HumanAddr },
    ChangeConfig(NewConfig),
    RegisterQuestContract { address: HumanAddr, quest: u8 },
    RegisterAirdropContract { contract: SecretContract },
    RemoveQuestContract { address: HumanAddr },
    ChangeQuestWeight { quest: u8, weight: u8 },
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    use HandleMsg::*;

    let config = Config::load(&deps.storage)?;

    match msg {
        CompleteQuest { address } => complete_quest(deps, env, config, address),
        // admin commands
        ChangeConfig(new_config) => change_config(deps, env, config, new_config),
        RegisterQuestContract { address, quest } => {
            register_quest_contract(deps, env, config, address, quest)
        }
        RegisterAirdropContract { contract } => {
            register_airdrop_contract(deps, env, config, contract)
        }
        ChangeQuestWeight { quest, weight } => {
            change_quest_weight(deps, env, config, quest, weight)
        }
        RemoveQuestContract { address } => remove_quest_contract(deps, env, config, address),
    }
}
