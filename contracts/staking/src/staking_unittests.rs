#![cfg(test)]

use cosmwasm_std::Uint128;
use cosmwasm_std::{to_binary, CosmosMsg, HumanAddr, StdResult, WasmMsg};
use secret_toolkit::utils::types::Contract;

use crate::msg::SubscriberMsg;

// copied from contract (where the function is and should stay private)
fn create_subscriber_msg(sub: Contract, user: &HumanAddr, new_stake: u128) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: sub.address,
        callback_code_hash: sub.hash,
        msg: to_binary(&SubscriberMsg::StakeChange {
            voter: user.clone(),
            new_stake: Uint128(new_stake),
        })?,
        send: vec![],
    }))
}

mod tests {
    use cosmwasm_std::StdError::Unauthorized;
    use cosmwasm_std::{
        from_binary, to_binary, Api, Extern, HumanAddr, Querier, StdResult, Storage, Uint128,
    };
    use secret_toolkit::snip20;
    use secret_toolkit::utils::types::Contract;

    use crate::constants::RESPONSE_BLOCK_SIZE;
    use crate::contract::query;
    use crate::msg::ResponseStatus::{NotChanged, Success};
    use crate::msg::{HandleAnswer, QueryAnswer, QueryMsg, ResponseStatus};
    use crate::staking_unittests::create_subscriber_msg;
    use crate::state::{
        BoosterItemInInventory, Config, Features, MultiplierContracts, ScheduleUnit,
    };
    use crate::unittest_utils::{
        add_multiplier_contracts_helper, add_subscriber_contracts_helper, apply_multiplier_helper,
        change_max_mul_config_helper, deposit_helper, drop_multiplier_helper,
        emergency_withdraw_helper, emergency_withdraw_skip_platform_helper,
        extract_generic_error_msg, init_helper, remove_multiplier_contracts_helper,
        remove_subscriber_contracts_helper, set_viewing_key_helper, unpause_feature_helper,
        withdraw_helper,
    };

    fn assert_total_locked<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        total_locked: u128,
        total_weight: u128,
    ) -> StdResult<()> {
        let query_response = query(&deps, QueryMsg::TotalLocked {})?;

        let total_locked_answer = from_binary::<QueryAnswer>(&query_response)?;
        assert_eq!(
            total_locked_answer,
            QueryAnswer::TotalLocked {
                amount: Uint128(total_locked),
                total_weight: Uint128(total_weight),
            }
        );

        Ok(())
    }

    fn assert_items<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        user: &str,
        key: &str,
        expected_items: Vec<BoosterItemInInventory>,
    ) -> StdResult<()> {
        let query_response = query(
            &deps,
            QueryMsg::BoosterItems {
                address: HumanAddr::from(user.to_string()),
                key: key.to_string(),
                page_number: None,
                page_size: 20,
            },
        )?;

        let items_answer = from_binary::<QueryAnswer>(&query_response)?;
        assert_eq!(
            items_answer,
            QueryAnswer::BoosterItems {
                items: expected_items
            }
        );

        Ok(())
    }

    fn assert_rewards_balance<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        user: &str,
        balance: u128,
        multiplier: u128,
        rewards: u128,
        height: u64,
    ) -> StdResult<QueryAnswer> {
        println!(
            "asserting rewards and balance of {} at height {}",
            user, height
        );
        let query_response = query(
            &deps,
            QueryMsg::Rewards {
                address: HumanAddr::from(user),
                key: "viewkey".to_string(),
                height: height,
            },
        )?;

        let rewards_answer = from_binary::<QueryAnswer>(&query_response)?;
        // println!("{:?}", rewards_answer);
        assert_eq!(
            rewards_answer,
            QueryAnswer::Rewards {
                rewards: Uint128(rewards)
            }
        );

        let query_response = query(
            &deps,
            QueryMsg::Balance {
                address: HumanAddr::from(user),
                key: "viewkey".to_string(),
            },
        )?;

        let answer = from_binary::<QueryAnswer>(&query_response)?;
        if let QueryAnswer::Balance {
            amount,
            total_multiplier,
            ..
        } = answer
        {
            assert_eq!(amount, Uint128(balance));
            assert_eq!(total_multiplier, Uint128(multiplier));
        } else {
            panic!("wrong queryAnswer variant");
        }

        Ok(answer)
    }

    #[test]
    fn test_unauthorized_update_multiplier_contracts() -> StdResult<()> {
        let mut deps = init_helper(None, None, None, None)?;
        let result = add_multiplier_contracts_helper(
            &mut deps,
            "non-admin",
            vec![HumanAddr::from("my-contract")],
        );

        assert_eq!(extract_generic_error_msg(result), "not an admin: non-admin");

        let result = remove_multiplier_contracts_helper(&mut deps, "non-admin2", vec![]);

        assert_eq!(
            extract_generic_error_msg(result),
            "not an admin: non-admin2"
        );

        Ok(())
    }

    #[test]
    fn test_add_remove_multiplier_contracts() -> StdResult<()> {
        let mut deps = init_helper(None, None, None, None)?;

        let is_allowed_res = MultiplierContracts::require_multiplier_contract(
            &deps.storage,
            &HumanAddr::from("NFTcontract1"),
        );
        assert!(is_allowed_res.is_err());
        assert_eq!(
            extract_generic_error_msg(is_allowed_res),
            "address NFTcontract1 is not allowed to set multipliers"
        );

        let result = add_multiplier_contracts_helper(
            &mut deps,
            "admin",
            vec![
                HumanAddr::from("NFTcontract1"),
                HumanAddr::from("NFTcontract2"),
            ],
        )?;

        assert!(matches!(
            result,
            HandleAnswer::AddMultiplierContracts {
                status: ResponseStatus::Success
            }
        ));

        let query_answer = query(
            &deps,
            QueryMsg::MultiplierContracts {
                page_number: None,
                page_size: 3,
            },
        )?;

        assert_eq!(
            from_binary::<QueryAnswer>(&query_answer)?,
            QueryAnswer::MultiplierContracts {
                contracts: vec![
                    HumanAddr::from("NFTcontract1"),
                    HumanAddr::from("NFTcontract2"),
                ]
            }
        );

        MultiplierContracts::require_multiplier_contract(
            &deps.storage,
            &HumanAddr::from("NFTcontract1"),
        )?;

        MultiplierContracts::require_multiplier_contract(
            &deps.storage,
            &HumanAddr::from("NFTcontract2"),
        )?;

        let result = remove_multiplier_contracts_helper(
            &mut deps,
            "admin",
            vec![HumanAddr::from("NFTcontract1")],
        )?;

        let query_answer = query(
            &deps,
            QueryMsg::MultiplierContracts {
                page_number: None,
                page_size: 3,
            },
        )?;

        assert_eq!(
            from_binary::<QueryAnswer>(&query_answer)?,
            QueryAnswer::MultiplierContracts {
                contracts: vec![HumanAddr::from("NFTcontract2")]
            }
        );

        assert!(matches!(
            result,
            HandleAnswer::RemoveMultiplierContracts {
                status: ResponseStatus::Success
            }
        ));

        let is_allowed_res = MultiplierContracts::require_multiplier_contract(
            &deps.storage,
            &HumanAddr::from("NFTcontract1"),
        );
        assert!(is_allowed_res.is_err());

        MultiplierContracts::require_multiplier_contract(
            &deps.storage,
            &HumanAddr::from("NFTcontract2"),
        )?;

        Ok(())
    }

    #[test]
    fn test_unauthorized_update_subscriber_contracts() -> StdResult<()> {
        let mut deps = init_helper(None, None, None, None)?;

        let result = add_subscriber_contracts_helper(
            &mut deps,
            "non-admin",
            vec![Contract {
                address: HumanAddr::from("my-contract"),
                hash: "".to_string(),
            }],
        );

        assert_eq!(extract_generic_error_msg(result), "not an admin: non-admin");

        let result = remove_subscriber_contracts_helper(&mut deps, "non-admin2", vec![]);

        assert_eq!(
            extract_generic_error_msg(result),
            "not an admin: non-admin2"
        );

        Ok(())
    }

    #[test]
    fn test_add_remove_subs() -> StdResult<()> {
        let sub_a = Contract {
            address: HumanAddr("sub_a".to_string()),
            hash: "".to_string(),
        };
        let sub_b = Contract {
            address: HumanAddr("sub_b".to_string()),
            hash: "".to_string(),
        };
        let sub_c = Contract {
            address: HumanAddr("sub_c".to_string()),
            hash: "".to_string(),
        };

        let mut deps = init_helper(Some(vec![sub_a.clone(), sub_b.clone()]), None, None, None)?;

        deposit_helper(&mut deps, "user", 100, None)?;
        add_subscriber_contracts_helper(&mut deps, "admin", vec![sub_c.clone()])?;

        let (messages, answer) = deposit_helper(&mut deps, "user".into(), 100, None)?;

        assert_eq!(
            messages,
            vec![
                create_subscriber_msg(sub_a.clone(), &HumanAddr("user".to_string()), 200).unwrap(),
                create_subscriber_msg(sub_b.clone(), &HumanAddr("user".to_string()), 200).unwrap(),
                create_subscriber_msg(sub_c.clone(), &HumanAddr("user".to_string()), 200).unwrap()
            ],
        );

        assert_eq!(answer, HandleAnswer::Deposit { status: Success });

        remove_subscriber_contracts_helper(
            &mut deps,
            "admin",
            vec![sub_a.address, sub_b.address, sub_c.address],
        )?;

        let (messages, answer) = withdraw_helper(&mut deps, "user".into(), 150, None)?;
        let config: Config = Config::load(&deps.storage)?;
        assert_eq!(
            messages,
            vec![snip20::send_msg(
                HumanAddr::from("platform"),
                Uint128(150),
                Some(to_binary(&platform::msg::ReceiveMsg::Deposit {
                    to: HumanAddr::from("user"),
                })?),
                None,
                None,
                RESPONSE_BLOCK_SIZE,
                config.token.hash,
                config.token.address,
            )?]
        );
        assert_eq!(answer, HandleAnswer::Redeem { status: Success });

        Ok(())
    }

    #[test]
    fn test_sanity() -> StdResult<()> {
        let mut deps = init_helper(
            None,
            None,
            Some(vec![
                ScheduleUnit::new(10, 5000),
                ScheduleUnit::new(20, 100),
            ]),
            None,
        )?;

        let query_answer_rewards = query(
            &deps,
            QueryMsg::Rewards {
                address: HumanAddr::from("user"),
                key: "viewkey".to_string(),
                height: 0,
            },
        );

        let query_answer_balance = query(
            &deps,
            QueryMsg::Balance {
                address: HumanAddr::from("user"),
                key: "viewkey".to_string(),
            },
        );

        let query_answer_items = query(
            &deps,
            QueryMsg::BoosterItems {
                address: HumanAddr::from("user"),
                key: "viewkey".to_string(),
                page_number: None,
                page_size: 30,
            },
        );

        assert_eq!(
            query_answer_rewards.unwrap_err(),
            Unauthorized { backtrace: None }
        );
        assert_eq!(
            query_answer_balance.unwrap_err(),
            Unauthorized { backtrace: None }
        );
        assert_eq!(
            query_answer_items.unwrap_err(),
            Unauthorized { backtrace: None }
        );

        set_viewing_key_helper(&mut deps, "user", "viewkey")?;
        set_viewing_key_helper(&mut deps, "whale", "viewkey")?;
        assert_rewards_balance(&deps, "user", 0, 100_000, 0, 0)?;
        assert_rewards_balance(&deps, "whale", 0, 100_000, 0, 0)?;
        assert_total_locked(&deps, 0, 0)?;

        let (messages, answer) = deposit_helper(&mut deps, "user", 75, Some(1))?;
        assert_eq!(messages, vec![]); // residue not yet rewarded since locked was 0 before
        assert_eq!(answer, HandleAnswer::Deposit { status: Success });
        assert_rewards_balance(&deps, "user", 75, 100_000, 0, 1)?;
        assert_rewards_balance(&deps, "user", 75, 100_000, 9999, 2)?;
        assert_total_locked(&deps, 75, 75_00_000)?;

        let (messages, answer) = deposit_helper(&mut deps, "whale", 675, Some(3))?;
        assert_eq!(messages, vec![]);
        assert_eq!(answer, HandleAnswer::Deposit { status: Success });
        assert_rewards_balance(&deps, "whale", 675, 100_000, 0, 3)?;
        assert_total_locked(&deps, 750, 750_00_000)?;

        assert_rewards_balance(&deps, "user", 75, 100_000, 15499, 4)?;
        assert_rewards_balance(&deps, "whale", 675, 100_000, 4499, 4)?;
        let (messages, answer) = withdraw_helper(&mut deps, "user", 50, Some(4))?;
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            snip20::send_msg(
                HumanAddr::from("platform"),
                Uint128(15499 + 50), // rewards + withdrawn amount
                Some(to_binary(&platform::msg::ReceiveMsg::Deposit {
                    to: HumanAddr::from("user")
                })?),
                None,
                None,
                RESPONSE_BLOCK_SIZE,
                "".to_string(),
                HumanAddr::from("token"),
            )?,
        );
        assert_eq!(answer, HandleAnswer::Redeem { status: Success });
        assert_rewards_balance(&deps, "user", 25, 100_000, 0, 4)?;
        assert_total_locked(&deps, 700, 700_00_000)?;

        // withdraw after inflation ends
        assert_rewards_balance(&deps, "whale", 675, 100_000, 34392, 25)?;
        let (messages, answer) = withdraw_helper(&mut deps, "whale", 675, Some(25))?;
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            snip20::send_msg(
                HumanAddr::from("platform"),
                Uint128(34392 + 675), // rewards + withdrawn amount
                Some(to_binary(&platform::msg::ReceiveMsg::Deposit {
                    to: HumanAddr::from("whale")
                })?),
                None,
                None,
                RESPONSE_BLOCK_SIZE,
                "".to_string(),
                HumanAddr::from("token"),
            )?,
        );
        assert_eq!(answer, HandleAnswer::Redeem { status: Success });
        assert_rewards_balance(&deps, "whale", 0, 100_000, 0, 25)?;
        assert_total_locked(&deps, 25, 25_00_000)?;

        Ok(())
    }

    #[test]
    fn test_init_max_multiplier_too_small() -> StdResult<()> {
        let result = init_helper(None, None, None, Some(1337));
        assert!(result.is_err());
        assert_eq!(
            extract_generic_error_msg(result),
            "max_multiplier can't be less than 100000",
        );

        Ok(())
    }

    #[test]
    fn test_multipliers() -> StdResult<()> {
        let mut deps = init_helper(None, None, Some(vec![ScheduleUnit::new(10, 5000)]), None)?;

        add_multiplier_contracts_helper(
            &mut deps,
            "admin",
            vec![HumanAddr::from("nft-contract1")],
        )?;

        set_viewing_key_helper(&mut deps, "user", "viewkey")?;
        set_viewing_key_helper(&mut deps, "whale", "viewkey")?;
        assert_rewards_balance(&deps, "user", 0, 100_000, 0, 0)?;
        assert_rewards_balance(&deps, "whale", 0, 100_000, 0, 0)?;
        assert_total_locked(&deps, 0, 0)?;
        assert_items(&deps, "whale", "viewkey", vec![])?;
        assert_items(&deps, "user", "viewkey", vec![])?;

        let (messages, answer) = deposit_helper(&mut deps, "user", 40, Some(1))?;
        assert_eq!(messages, vec![]); // residue not yet rewarded since locked was 0 before
        assert_eq!(answer, HandleAnswer::Deposit { status: Success });
        let (messages, answer) = deposit_helper(&mut deps, "whale", 160, Some(1))?;
        assert_eq!(messages, vec![]); // residue not yet rewarded since locked was 0 before
        assert_eq!(answer, HandleAnswer::Deposit { status: Success });

        assert_rewards_balance(&deps, "user", 40, 100_000, 0, 1)?;
        assert_rewards_balance(&deps, "whale", 160, 100_000, 0, 1)?;
        assert_rewards_balance(&deps, "user", 40, 100_000, 2000, 2)?;
        assert_rewards_balance(&deps, "whale", 160, 100_000, 8000, 2)?;
        assert_total_locked(&deps, 200, 200_00_000)?;

        let (messages, answer) = apply_multiplier_helper(
            &mut deps,
            "nft-contract1",
            150_000, // 150% multiplier - causes weight to be 60
            Some(1),
            "user's-item-1.5x",
            "user",
        )?;
        assert_eq!(messages, vec![]);
        assert_eq!(answer, HandleAnswer::ApplyMultiplier { status: Success });
        assert_rewards_balance(&deps, "user", 40, 150_000, 2727, 2)?;
        assert_rewards_balance(&deps, "whale", 160, 100_000, 7272, 2)?;
        assert_total_locked(&deps, 200, 220_00_000)?;
        assert_items(
            &deps,
            "user",
            "viewkey",
            vec![BoosterItemInInventory {
                multiplier: 150_000,
                contract: HumanAddr::from("nft-contract1"),
                id: "user's-item-1.5x".to_string(),
            }],
        )?;
        assert_items(&deps, "whale", "viewkey", vec![])?;

        let (messages, answer) = apply_multiplier_helper(
            &mut deps,
            "nft-contract1",
            133_333, // 133.333% multiplier - causes weight to be 79
            Some(1),
            "user's-item-133%",
            "user",
        )?;
        assert_eq!(messages, vec![]);
        assert_eq!(answer, HandleAnswer::ApplyMultiplier { status: Success });

        assert_rewards_balance(&deps, "user", 40, 183_333, 3142, 2)?;
        assert_rewards_balance(&deps, "whale", 160, 100_000, 6857, 2)?;
        assert_total_locked(&deps, 200, 233_00_000)?;
        assert_items(
            &deps,
            "user",
            "viewkey",
            vec![
                BoosterItemInInventory {
                    multiplier: 150_000,
                    contract: HumanAddr::from("nft-contract1"),
                    id: "user's-item-1.5x".to_string(),
                },
                BoosterItemInInventory {
                    multiplier: 133_333,
                    contract: HumanAddr::from("nft-contract1"),
                    id: "user's-item-133%".to_string(),
                },
            ],
        )?;
        assert_items(&deps, "whale", "viewkey", vec![])?;

        let (messages, answer) = withdraw_helper(&mut deps, "user", 20, Some(2))?;
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            snip20::send_msg(
                HumanAddr::from("platform"),
                Uint128(3142 + 20), // rewards + withdrawn amount
                Some(to_binary(&platform::msg::ReceiveMsg::Deposit {
                    to: HumanAddr::from("user")
                })?),
                None,
                None,
                RESPONSE_BLOCK_SIZE,
                "".to_string(),
                HumanAddr::from("token"),
            )?,
        );
        assert_eq!(answer, HandleAnswer::Redeem { status: Success });
        assert_rewards_balance(&deps, "user", 20, 183_333, 0, 2)?;
        assert_rewards_balance(&deps, "whale", 160, 100_000, 6857, 2)?;
        assert_rewards_balance(&deps, "user", 20, 183_333, 932, 3)?;
        assert_rewards_balance(&deps, "whale", 160, 100_000, 4067 + 6857, 3)?;
        assert_total_locked(&deps, 180, 196_00_000)?;

        // whale deposits some more
        let (messages, answer) = deposit_helper(&mut deps, "whale", 40, Some(3))?;
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            snip20::send_msg(
                HumanAddr::from("platform"),
                Uint128(4067 + 6857), // rewards
                Some(to_binary(&platform::msg::ReceiveMsg::Deposit {
                    to: HumanAddr::from("whale")
                })?),
                None,
                None,
                RESPONSE_BLOCK_SIZE,
                "".to_string(),
                HumanAddr::from("token"),
            )?,
        );
        assert_eq!(answer, HandleAnswer::Deposit { status: Success });
        assert_rewards_balance(&deps, "whale", 200, 100_000, 0, 3)?;
        assert_rewards_balance(&deps, "user", 20, 183_333, 932 + 774 + 1, 4)?;
        assert_rewards_balance(&deps, "whale", 200, 100_000, 4225, 4)?;
        assert_total_locked(&deps, 220, 236_00_000)?;

        // user unlocks the 150% item, now has only 133%
        let (messages, answer) = drop_multiplier_helper(
            &mut deps,
            "nft-contract1",
            Some(4),
            "user's-item-1.5x",
            "user",
        )?;
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            snip20::send_msg(
                HumanAddr::from("platform"),
                Uint128(932 + 774 + 1), // rewards
                Some(to_binary(&platform::msg::ReceiveMsg::Deposit {
                    to: HumanAddr::from("user")
                })?),
                None,
                None,
                RESPONSE_BLOCK_SIZE,
                "".to_string(),
                HumanAddr::from("token"),
            )?,
        );
        assert_eq!(answer, HandleAnswer::DropMultiplier { status: Success });
        assert_items(
            &deps,
            "user",
            "viewkey",
            vec![BoosterItemInInventory {
                multiplier: 133_333,
                contract: HumanAddr::from("nft-contract1"),
                id: "user's-item-133%".to_string(),
            }],
        )?;
        assert_items(&deps, "whale", "viewkey", vec![])?;

        assert_rewards_balance(&deps, "whale", 200, 100_000, 4225, 4)?;
        assert_rewards_balance(&deps, "user", 20, 133_333, 0, 4)?;
        assert_rewards_balance(&deps, "whale", 200, 100_000, 4225 + 4411 + 1, 5)?;
        assert_rewards_balance(&deps, "user", 20, 133_333, 588, 5)?;
        assert_total_locked(&deps, 220, 226_00_000)?;

        let (messages, answer) = apply_multiplier_helper(
            &mut deps,
            "nft-contract1",
            125_000, // 125% multiplier - causes weight to be 250
            Some(5),
            "whales's-item-125%",
            "whale",
        )?;
        assert_items(
            &deps,
            "user",
            "viewkey",
            vec![BoosterItemInInventory {
                multiplier: 133_333,
                contract: HumanAddr::from("nft-contract1"),
                id: "user's-item-133%".to_string(),
            }],
        )?;
        assert_items(
            &deps,
            "whale",
            "viewkey",
            vec![BoosterItemInInventory {
                multiplier: 125_000,
                contract: HumanAddr::from("nft-contract1"),
                id: "whales's-item-125%".to_string(),
            }],
        )?;
        assert_eq!(answer, HandleAnswer::ApplyMultiplier { status: Success });
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            snip20::send_msg(
                HumanAddr::from("platform"),
                Uint128(4225 + 4411 + 1), // rewards
                Some(to_binary(&platform::msg::ReceiveMsg::Deposit {
                    to: HumanAddr::from("whale")
                })?),
                None,
                None,
                RESPONSE_BLOCK_SIZE,
                "".to_string(),
                HumanAddr::from("token"),
            )?,
        );

        assert_rewards_balance(&deps, "whale", 200, 125_000, 0, 5)?;
        assert_rewards_balance(&deps, "whale", 200, 125_000, 4518, 6)?;
        assert_rewards_balance(&deps, "user", 20, 133_333, 588 + 481 + 1, 6)?;
        assert_total_locked(&deps, 220, 276_00_000)?;

        // return to original multipilers
        let (messages, answer) = drop_multiplier_helper(
            &mut deps,
            "nft-contract1",
            Some(6),
            "user's-item-133%",
            "user",
        )?;
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            snip20::send_msg(
                HumanAddr::from("platform"),
                Uint128(588 + 481 + 1), // rewards
                Some(to_binary(&platform::msg::ReceiveMsg::Deposit {
                    to: HumanAddr::from("user")
                })?),
                None,
                None,
                RESPONSE_BLOCK_SIZE,
                "".to_string(),
                HumanAddr::from("token"),
            )?,
        );
        assert_eq!(answer, HandleAnswer::DropMultiplier { status: Success });
        assert_total_locked(&deps, 220, 270_00_000)?;

        let (messages, answer) = drop_multiplier_helper(
            &mut deps,
            "nft-contract1",
            Some(6),
            "whales's-item-125%",
            "whale",
        )?;
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            snip20::send_msg(
                HumanAddr::from("platform"),
                Uint128(4518), // rewards
                Some(to_binary(&platform::msg::ReceiveMsg::Deposit {
                    to: HumanAddr::from("whale")
                })?),
                None,
                None,
                RESPONSE_BLOCK_SIZE,
                "".to_string(),
                HumanAddr::from("token"),
            )?,
        );
        assert_eq!(answer, HandleAnswer::DropMultiplier { status: Success });

        assert_rewards_balance(&deps, "user", 20, 100_000, 0, 6)?;
        assert_rewards_balance(&deps, "whale", 200, 100_000, 0, 6)?;
        assert_rewards_balance(&deps, "user", 20, 100_000, 454 + 1, 7)?;
        assert_rewards_balance(&deps, "whale", 200, 100_000, 4546, 7)?;
        assert_total_locked(&deps, 220, 220_00_000)?;
        assert_items(&deps, "user", "viewkey", vec![])?;
        assert_items(&deps, "whale", "viewkey", vec![])?;

        // test emergency withdraws:
        unpause_feature_helper(&mut deps, Features::EmergencyWithdraw)?;
        let (messages, answer) = emergency_withdraw_helper(&mut deps, "user")?;
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            snip20::send_msg(
                HumanAddr::from("platform"),
                Uint128(20),
                Some(to_binary(&platform::msg::ReceiveMsg::Deposit {
                    to: HumanAddr::from("user")
                })?),
                None,
                None,
                RESPONSE_BLOCK_SIZE,
                "".to_string(),
                HumanAddr::from("token"),
            )?,
        );
        assert_eq!(answer, HandleAnswer::EmergencyWithdraw { status: Success });
        assert_rewards_balance(&deps, "user", 0, 100_000, 0, 7)?;

        unpause_feature_helper(&mut deps, Features::EmergencyWithdrawSkipPlatform)?;
        let (messages, answer) = emergency_withdraw_skip_platform_helper(&mut deps, "whale")?;
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            snip20::transfer_msg(
                HumanAddr::from("whale"),
                Uint128(200),
                None,
                None,
                RESPONSE_BLOCK_SIZE,
                "".to_string(),
                HumanAddr::from("token"),
            )?,
        );
        assert_eq!(
            answer,
            HandleAnswer::EmergencyWithdrawSkipPlatform { status: Success }
        );
        assert_rewards_balance(&deps, "whale", 0, 100_000, 0, 7)?;

        Ok(())
    }

    #[test]
    fn test_wrong_contract_apply_multiplier() -> StdResult<()> {
        let mut deps = init_helper(None, None, None, None)?;

        let result = apply_multiplier_helper(
            &mut deps,
            "unknown-contract",
            150_000,
            Some(1),
            "user's-item-1.5x",
            "user",
        );
        assert!(result.is_err());
        assert_eq!(
            extract_generic_error_msg(result),
            "address unknown-contract is not allowed to set multipliers".to_string()
        );

        Ok(())
    }

    #[test]
    fn test_wrong_contract_drop_multiplier() -> StdResult<()> {
        let mut deps = init_helper(None, None, None, None)?;

        let result = drop_multiplier_helper(
            &mut deps,
            "unknown-contract",
            Some(1),
            "user's-item-1.5x",
            "user",
        );
        assert!(result.is_err());
        assert_eq!(
            extract_generic_error_msg(result),
            "address unknown-contract is not allowed to set multipliers".to_string()
        );

        Ok(())
    }

    #[test]
    fn test_apply_multiplier_to_unexistent_balance() -> StdResult<()> {
        let mut deps = init_helper(None, None, Some(vec![ScheduleUnit::new(10, 5000)]), None)?;

        add_multiplier_contracts_helper(
            &mut deps,
            "admin",
            vec![HumanAddr::from("nft-contract1")],
        )?;

        apply_multiplier_helper(
            &mut deps,
            "nft-contract1",
            200_000,
            Some(0),
            "user's-item-2x",
            "user",
        )?;

        set_viewing_key_helper(&mut deps, "user", "viewkey")?;
        set_viewing_key_helper(&mut deps, "whale", "viewkey")?;
        assert_rewards_balance(&deps, "user", 0, 200_000, 0, 0)?;
        assert_rewards_balance(&deps, "whale", 0, 100_000, 0, 0)?;

        deposit_helper(&mut deps, "user", 1, Some(1))?;
        deposit_helper(&mut deps, "whale", 2, Some(1))?;

        assert_rewards_balance(&deps, "user", 1, 200_000, 5000, 2)?;
        assert_rewards_balance(&deps, "whale", 2, 100_000, 5000, 2)?;

        Ok(())
    }

    #[test]
    fn test_apply_locked_item() -> StdResult<()> {
        let mut deps = init_helper(None, None, Some(vec![ScheduleUnit::new(10, 5000)]), None)?;

        add_multiplier_contracts_helper(
            &mut deps,
            "admin",
            vec![
                HumanAddr::from("nft-contract1"),
                HumanAddr::from("nft-contract2"),
            ],
        )?;

        apply_multiplier_helper(
            &mut deps,
            "nft-contract1",
            200_000,
            Some(0),
            "user's-item-2x",
            "user",
        )?;

        let (messages, answer) = apply_multiplier_helper(
            &mut deps,
            "nft-contract1",
            200_000,
            Some(0),
            "user's-item-2x",
            "whale",
        )?;

        assert_eq!(messages, vec![]);
        assert_eq!(answer, HandleAnswer::ApplyMultiplier { status: NotChanged });

        // this one should work because it's from another contract
        let (_, answer) = apply_multiplier_helper(
            &mut deps,
            "nft-contract2",
            200_000,
            Some(0),
            "user's-item-2x",
            "whale",
        )?;

        assert_eq!(answer, HandleAnswer::ApplyMultiplier { status: Success });
        Ok(())
    }

    #[test]
    fn test_drop_unlocked_item() -> StdResult<()> {
        let mut deps = init_helper(None, None, Some(vec![ScheduleUnit::new(10, 5000)]), None)?;

        add_multiplier_contracts_helper(
            &mut deps,
            "admin",
            vec![HumanAddr::from("nft-contract1")],
        )?;

        let (messages, answer) = drop_multiplier_helper(
            &mut deps,
            "nft-contract1",
            Some(0),
            "user's-item-2x",
            "user",
        )?;

        assert_eq!(answer, HandleAnswer::DropMultiplier { status: NotChanged });
        assert_eq!(messages.len(), 0);

        Ok(())
    }

    #[test]
    fn test_drop_item_belonging_to_another() -> StdResult<()> {
        let mut deps = init_helper(None, None, Some(vec![ScheduleUnit::new(10, 5000)]), None)?;

        add_multiplier_contracts_helper(
            &mut deps,
            "admin",
            vec![HumanAddr::from("nft-contract1")],
        )?;

        apply_multiplier_helper(
            &mut deps,
            "nft-contract1",
            200_000,
            Some(0),
            "user's-item-2x",
            "user",
        )?;

        let result = drop_multiplier_helper(
            &mut deps,
            "nft-contract1",
            Some(0),
            "user's-item-2x",
            "another-user",
        );

        assert!(result.is_err());
        assert_eq!(
            extract_generic_error_msg(result),
            "Item user's-item-2x was not locked by another-user".to_string()
        );

        Ok(())
    }

    #[test]
    fn test_total_locked_lsd_obfuscation() -> StdResult<()> {
        let mut deps = init_helper(None, None, None, None)?;

        deposit_helper(&mut deps, "user", 1234567899876543, Some(1))?;
        assert_total_locked(&deps, 1230000000000000, 1230000000000000_00_000)?;

        add_multiplier_contracts_helper(&mut deps, "admin", vec![HumanAddr::from("c")])?;
        apply_multiplier_helper(&mut deps, "c", 200_000, Some(0), "id", "user")?;
        assert_total_locked(&deps, 1230000000000000, 2460000000000000_00_000)?;

        Ok(())
    }

    #[test]
    fn test_change_max_multiplier_config_smaller_than_one() -> StdResult<()> {
        let mut deps = init_helper(
            None,
            None,
            Some(vec![ScheduleUnit::new(10, 10_000)]),
            Some(200_000),
        )?;

        let result = change_max_mul_config_helper(&mut deps, 10);
        assert_eq!(
            extract_generic_error_msg(result),
            "max multiplier cannot be smaller than 1".to_string()
        );
        Ok(())
    }

    #[test]
    fn test_total_max_multiplier() -> StdResult<()> {
        let mut deps = init_helper(
            None,
            None,
            Some(vec![ScheduleUnit::new(10, 10_000)]),
            Some(200_000),
        )?;

        set_viewing_key_helper(&mut deps, "user", "viewkey")?;
        deposit_helper(&mut deps, "user", 2222, Some(0))?;
        deposit_helper(&mut deps, "whale", 1111, Some(0))?;

        add_multiplier_contracts_helper(&mut deps, "admin", vec![HumanAddr::from("c")])?;
        apply_multiplier_helper(&mut deps, "c", 150_000, Some(0), "id", "user")?;
        assert_total_locked(&deps, 3330, 4440_00_000)?;

        apply_multiplier_helper(&mut deps, "c", 175_000, Some(0), "id2", "user")?;
        assert_total_locked(&deps, 3330, 5550_00_000)?; // hit max
        let balance_answer = assert_rewards_balance(&deps, "user", 2222, 225_000, 0, 0)?;
        if let QueryAnswer::Balance {
            effective_multiplier,
            ..
        } = balance_answer
        {
            assert_eq!(effective_multiplier, Uint128(200_000));
        } else {
            panic!("wrong queryAnswer variant");
        }
        assert_rewards_balance(&deps, "user", 2222, 225_000, 7999, 1)?;

        drop_multiplier_helper(&mut deps, "c", Some(1), "id2", "user")?;
        assert_total_locked(&deps, 3330, 4440_00_000)?;
        let balance_answer = assert_rewards_balance(&deps, "user", 2222, 150_000, 7500, 2)?;
        if let QueryAnswer::Balance {
            effective_multiplier,
            ..
        } = balance_answer
        {
            assert_eq!(effective_multiplier, Uint128(150_000));
        } else {
            panic!("wrong queryAnswer variant");
        }

        Ok(())
    }

    #[test]
    fn test_max_multiplier_changed() -> StdResult<()> {
        let mut deps = init_helper(
            None,
            None,
            Some(vec![ScheduleUnit::new(10, 10_000)]),
            Some(200_000),
        )?;

        set_viewing_key_helper(&mut deps, "whale", "viewkey")?;
        set_viewing_key_helper(&mut deps, "user", "viewkey")?;
        deposit_helper(&mut deps, "whale", 2222, Some(0))?;
        deposit_helper(&mut deps, "user", 1111, Some(0))?;

        add_multiplier_contracts_helper(&mut deps, "admin", vec![HumanAddr::from("c")])?;
        apply_multiplier_helper(&mut deps, "c", 150_000, Some(0), "id", "whale")?;
        assert_total_locked(&deps, 3330, 4440_00_000)?;

        apply_multiplier_helper(&mut deps, "c", 175_000, Some(0), "id2", "whale")?;
        assert_total_locked(&deps, 3330, 5550_00_000)?; // hit max
        let balance_answer = assert_rewards_balance(&deps, "whale", 2222, 225_000, 0, 0)?;
        if let QueryAnswer::Balance {
            effective_multiplier,
            ..
        } = balance_answer
        {
            assert_eq!(effective_multiplier, Uint128(200_000));
        } else {
            panic!("wrong queryAnswer variant");
        }

        assert_rewards_balance(&deps, "whale", 2222, 225_000, 7999, 1)?;

        change_max_mul_config_helper(&mut deps, 300_000)?;

        // still reflects previous max_multiplier
        let balance_answer = assert_rewards_balance(&deps, "whale", 2222, 225_000, 7999, 1)?;
        if let QueryAnswer::Balance {
            effective_multiplier,
            ..
        } = balance_answer
        {
            assert_eq!(effective_multiplier, Uint128(200_000));
        } else {
            panic!("wrong queryAnswer variant");
        }

        // should apply new config to user after this withdraw
        withdraw_helper(&mut deps, "whale", 0, Some(0))?;
        let balance_answer = assert_rewards_balance(&deps, "whale", 2222, 225_000, 8181, 1)?;
        if let QueryAnswer::Balance {
            effective_multiplier,
            ..
        } = balance_answer
        {
            assert_eq!(effective_multiplier, Uint128(225_000));
        } else {
            panic!("wrong queryAnswer variant");
        }
        assert_total_locked(&deps, 3330, 4990_00_000 + 1120_00_000)?;

        // verify other user doesn't change
        let balance_answer = assert_rewards_balance(&deps, "user", 1111, 100_000, 1818, 1)?;
        if let QueryAnswer::Balance {
            effective_multiplier,
            ..
        } = balance_answer
        {
            assert_eq!(effective_multiplier, Uint128(100_000));
        } else {
            panic!("wrong queryAnswer variant");
        }

        Ok(())
    }
}
