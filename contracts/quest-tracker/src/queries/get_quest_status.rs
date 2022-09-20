use crate::queries::QuestStatusResponse;
use crate::state::get_quest_status;
use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdResult, Storage};

pub fn query_get_quest_status<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> StdResult<QuestStatusResponse> {
    let quest = get_quest_status(&deps.storage, &address);

    Ok(QuestStatusResponse { quest })
}
