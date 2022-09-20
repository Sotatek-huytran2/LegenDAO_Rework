use schemars::JsonSchema;
use serde::Deserialize;

use crate::state::Config;
use cosmwasm_std::{Api, Env, Extern, HumanAddr, InitResponse, Querier, StdResult, Storage};

use crate::types::secret_contract::SecretContract;

#[derive(Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub platform: SecretContract,
    pub token: SecretContract,
    pub quest_contract: Option<HumanAddr>,
}

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let clone_sender = || env.message.sender.clone();
    let admin = msg.admin.clone().unwrap_or_else(clone_sender);
    let confirmer = msg.admin.unwrap_or_else(clone_sender);

    Config::new(
        admin,
        confirmer,
        msg.platform,
        msg.token,
        msg.quest_contract,
    )
    .save(&mut deps.storage)?;

    Ok(InitResponse::default())
}
