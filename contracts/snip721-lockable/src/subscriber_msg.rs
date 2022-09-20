use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    ApplyMultiplier {
        to: HumanAddr,
        multiplier: u32,
        item_id: String,
    },
    DropMultiplier {
        from: HumanAddr,
        item_id: String,
    },
}
