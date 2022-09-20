use cosmwasm_std::{Api, Env, Extern, HumanAddr, InitResponse, Querier, StdResult, Storage};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::state::{add_quest_contract, add_quest_weight, Config};
use crate::types::secret_contract::SecretContract;

#[derive(Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QuestContract {
    pub contract: HumanAddr,
    pub quest: u8,
}

#[derive(Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QuestWeight {
    pub quest: u8,
    pub weight: u8,
}

#[derive(Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub token: SecretContract,
    pub platform: SecretContract,
    pub quest_contracts: Vec<QuestContract>,
    pub quest_weights: Vec<QuestWeight>,
}

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let admin = msg.admin.unwrap_or(env.message.sender);
    let config = Config::new(admin, msg.token, msg.platform);

    config.save(&mut deps.storage)?;

    for quest_contract in msg.quest_contracts {
        add_quest_contract(
            &mut deps.storage,
            &quest_contract.contract,
            quest_contract.quest,
        )?;
    }

    for quest_weight in msg.quest_weights {
        add_quest_weight(&mut deps.storage, quest_weight.quest, quest_weight.weight)?;
    }

    Ok(InitResponse {
        messages: vec![],
        log: vec![],
    })
}
