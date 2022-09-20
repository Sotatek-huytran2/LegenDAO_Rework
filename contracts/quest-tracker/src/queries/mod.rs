mod get_quest_status;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_binary, Api, Binary, Extern, HumanAddr, Querier, StdResult, Storage};

use crate::queries::get_quest_status::query_get_quest_status;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetQuestStatus { address: HumanAddr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QuestStatusResponse {
    quest: u8,
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetQuestStatus { address } => to_binary(&query_get_quest_status(deps, address)?),
    }
}
