#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;
    use cosmwasm_std::{to_binary, HumanAddr, StdError, StdResult, Uint128};
    use secret_toolkit::viewing_key::{ViewingKey, ViewingKeyStore};

    use crate::auto_claim::AutoClaims;
    use crate::msg::{HandleAnswer, PlatformApi, ResponseStatus};
    use crate::state::{Features, SECONDS_IN_DAY};
    use crate::unittests_utils::{
        add_pauser, claim_json, create_bank_send_message, create_redeem_message,
        create_send_message, do_claim, do_deposit, do_redeem, do_send_from_platform,
        do_set_viewing_key, expect_platform_total_balance, extract_generic_error_msg,
        extract_messages, init_helper, mock_env_with_time, pause_feature, query_feature_status,
        query_is_pauser,
    };

    #[test]
    fn test_claims() -> StdResult<()> {
        let mut deps = init_helper(None, None)?;

        do_deposit(&mut deps, "bob", 1000, 2)?;
        do_deposit(&mut deps, "alice", 1000, 2)?;

        assert_eq!(AutoClaims::len(&deps.storage)?, 0);

        do_redeem(&mut deps, "bob", 1, SECONDS_IN_DAY)?; // Redeem 1
        do_redeem(&mut deps, "bob", 2, SECONDS_IN_DAY)?; // Redeem 2
        do_redeem(&mut deps, "alice", 3, SECONDS_IN_DAY)?; // Redeem 3
        do_redeem(&mut deps, "bob", 4, 3 * SECONDS_IN_DAY)?; // Redeem 4
        do_redeem(&mut deps, "alice", 5, 4 * SECONDS_IN_DAY)?; // Redeem 5
        do_redeem(&mut deps, "bob", 6, 5 * SECONDS_IN_DAY)?; // Redeem 6
        do_redeem(&mut deps, "alice", 7, 6 * SECONDS_IN_DAY)?; // Redeem 7
        assert_eq!(AutoClaims::len(&deps.storage)?, 6);

        // No redeems vested yet
        let result = AutoClaims::peek_next_claim(
            &deps.storage,
            &mock_env_with_time(HumanAddr::from("irrelevant"), 5 * SECONDS_IN_DAY),
        );
        assert!(result.as_ref().unwrap().is_none(), "{:?}", result);

        // Redeem 1+2
        let result = AutoClaims::peek_next_claim(
            &deps.storage,
            &mock_env_with_time(
                HumanAddr::from("irrelevant"),
                21 * SECONDS_IN_DAY + SECONDS_IN_DAY,
            ),
        );
        assert!(result.as_ref().unwrap().is_some(), "{:?}", result);

        // Claiming redeems 1+2, thus no additional claims (auto claim is discarded)
        let result_messages = do_claim(&mut deps, "bob", 21 * SECONDS_IN_DAY + SECONDS_IN_DAY + 1)?;
        assert_eq!(
            result_messages,
            vec![
                create_redeem_message(3)?,
                create_bank_send_message(MOCK_CONTRACT_ADDR, "bob", 3),
            ],
            "{:?}",
            result_messages
        );

        // Auto claim Redeem 3
        let result_messages = extract_messages(do_deposit(
            &mut deps,
            "bob",
            1000,
            21 * SECONDS_IN_DAY + SECONDS_IN_DAY + 1,
        )?);

        assert_eq!(
            result_messages,
            vec![
                create_redeem_message(3)?,
                create_bank_send_message(MOCK_CONTRACT_ADDR, "alice", 3),
            ],
            "{:?}",
            result_messages
        );

        // No more auto claims this day
        let result_messages = extract_messages(do_deposit(
            &mut deps,
            "bob",
            1000,
            21 * SECONDS_IN_DAY + SECONDS_IN_DAY + 1,
        )?);
        assert_eq!(result_messages, vec![], "{:?}", result_messages);

        // Redeems 4+6
        let result_messages =
            extract_messages(do_deposit(&mut deps, "bob", 1000, 27 * SECONDS_IN_DAY)?);
        assert_eq!(
            result_messages,
            vec![
                create_redeem_message(4 + 6)?,
                create_bank_send_message(MOCK_CONTRACT_ADDR, "bob", 4 + 6),
            ],
            "{:?}",
            result_messages
        );

        // Redeem 5
        let result_messages =
            extract_messages(do_deposit(&mut deps, "bob", 1000, 27 * SECONDS_IN_DAY)?);
        assert_eq!(
            result_messages,
            vec![
                create_redeem_message(5)?,
                create_bank_send_message(MOCK_CONTRACT_ADDR, "alice", 5),
            ],
            "{:?}",
            result_messages
        );

        // Auto claim of Redeem 6
        let result = AutoClaims::peek_next_claim(
            &deps.storage,
            &mock_env_with_time(HumanAddr::from("irrelevant"), 30 * SECONDS_IN_DAY),
        );
        let claim = result.as_ref().unwrap().as_ref();
        assert!(claim.is_some(), "{:?}", result);
        assert_eq!(
            serde_json::to_string(claim.unwrap()).unwrap(),
            claim_json("bob", 26 * SECONDS_IN_DAY),
            "{:?}",
            result
        );

        // Auto claim of Redeem 6 is discarded since already claimed
        let result_messages =
            extract_messages(do_deposit(&mut deps, "bob", 1000, 30 * SECONDS_IN_DAY)?);
        assert_eq!(result_messages, vec![], "{:?}", result_messages);

        // Redeem 7
        let result = AutoClaims::peek_next_claim(
            &deps.storage,
            &mock_env_with_time(HumanAddr::from("irrelevant"), 30 * SECONDS_IN_DAY),
        );
        let claim = result.as_ref().unwrap().as_ref();
        assert!(claim.is_some(), "{:?}", result);
        assert_eq!(
            serde_json::to_string(claim.unwrap()).unwrap(),
            claim_json("alice", 27 * SECONDS_IN_DAY),
            "{:?}",
            result
        );

        let result_messages =
            extract_messages(do_deposit(&mut deps, "bob", 1000, 30 * SECONDS_IN_DAY)?);
        assert_eq!(
            result_messages,
            vec![
                create_redeem_message(7)?,
                create_bank_send_message(MOCK_CONTRACT_ADDR, "alice", 7),
            ],
            "{:?}",
            result_messages
        );

        // No more pending redeems
        let result = AutoClaims::peek_next_claim(
            &deps.storage,
            &mock_env_with_time(HumanAddr::from("irrelevant"), 30 * SECONDS_IN_DAY),
        );
        assert!(result.as_ref().unwrap().is_none(), "{:?}", result);

        Ok(())
    }

    #[test]
    fn test_receiving() -> StdResult<()> {
        let mut deps = init_helper(
            Some(vec![
                HumanAddr::from("receiving1"),
                HumanAddr::from("receiving2"),
            ]),
            None,
        )?;

        do_deposit(&mut deps, "bob", 1000, 2)?;

        // 1st sending: first contract
        let result_messages = do_send_from_platform(
            &mut deps,
            "bob",
            HumanAddr::from("receiving1"),
            Some(Uint128::from(200u128)),
        )?;
        assert_eq!(
            result_messages,
            vec![create_send_message(
                "receiving1",
                200,
                Some(to_binary(&PlatformApi::ReceiveFromPlatform {
                    from: HumanAddr::from("bob"),
                    msg: Default::default(),
                })?),
            )?],
        );

        // 2nd sending: second contract
        let result_messages = do_send_from_platform(
            &mut deps,
            "bob",
            HumanAddr::from("receiving2"),
            Some(Uint128::from(200u128)),
        )?;
        assert_eq!(
            result_messages,
            vec![create_send_message(
                "receiving2",
                200,
                Some(to_binary(&PlatformApi::ReceiveFromPlatform {
                    from: HumanAddr::from("bob"),
                    msg: Default::default(),
                })?),
            )?],
        );

        // 3rd sending: too many funds
        assert!(do_send_from_platform(
            &mut deps,
            "bob",
            HumanAddr::from("receiving2"),
            Some(Uint128::from(2000u128)),
        )
        .is_err());

        // 4th sending: wrong receiving contract
        assert!(
            do_send_from_platform(&mut deps, "bob", HumanAddr::from("not receiving"), None)
                .is_err()
        );

        Ok(())
    }

    #[test]
    fn test_deposit_wrong_token() -> StdResult<()> {
        let mut deps = init_helper(None, Some(HumanAddr::from("another-token")))?;

        let result = do_deposit(&mut deps, "bob", 1000, 2);
        assert_eq!(
            extract_generic_error_msg(result),
            "this token is not supported. Supported: another-token, got: token",
        );

        Ok(())
    }

    #[test]
    fn test_set_viewing_key() -> StdResult<()> {
        const VIEWING_KEY: &str = "viewing key tests";

        let mut deps = init_helper(None, None)?;
        let result = do_set_viewing_key(&mut deps, "bob", VIEWING_KEY)?;

        match result {
            HandleAnswer::SetViewingKey { status } => {
                assert!(matches!(status, ResponseStatus::Success))
            }
            _ => panic!("unexpected handle answer variant"),
        }

        assert!(ViewingKey::check(&deps.storage, &HumanAddr::from("bob"), VIEWING_KEY).is_ok());
        assert!(ViewingKey::check(&deps.storage, &HumanAddr::from("bob"), "wrong").is_err());
        assert!(ViewingKey::check(&deps.storage, &HumanAddr::from("eve"), VIEWING_KEY).is_err());
        assert!(ViewingKey::check(&deps.storage, &HumanAddr::from("eve"), "wrong").is_err());

        Ok(())
    }

    #[test]
    fn test_total_balance_query() -> StdResult<()> {
        let mut deps = init_helper(None, None)?;

        expect_platform_total_balance(&deps, 0, 0)?;

        do_deposit(&mut deps, "bob", 1021, 2)?;
        // obfuscates all but first three digits
        expect_platform_total_balance(&deps, 1020, 0)?;

        do_deposit(&mut deps, "alice", 413555, 3)?;
        expect_platform_total_balance(&deps, 414000, 0)?;

        do_redeem(&mut deps, "alice", 4999, 4)?;
        expect_platform_total_balance(&deps, 409000, 4990)?;

        Ok(())
    }

    #[test]
    fn test_add_pauser_unauthorized() -> StdResult<()> {
        let mut deps = init_helper(None, None)?;
        let error = add_pauser(&mut deps, "bob", 3, Some("non-admin")).unwrap_err();
        assert_eq!(error, StdError::unauthorized());
        Ok(())
    }

    #[test]
    fn test_pause_feature_redeem() -> StdResult<()> {
        let mut deps = init_helper(None, None)?;

        let is_pauser = query_is_pauser(&deps, "bob")?;
        assert_eq!(
            is_pauser,
            "{\"is_pauser\":{\"is_pauser\":false}}".to_string()
        );

        let error = pause_feature(&mut deps, Features::Redeem, 2, "bob").unwrap_err();
        assert_eq!(error, StdError::unauthorized());

        add_pauser(&mut deps, "bob", 3, None)?;

        let is_pauser = query_is_pauser(&deps, "bob")?;
        assert_eq!(
            is_pauser,
            "{\"is_pauser\":{\"is_pauser\":true}}".to_string()
        );

        let redeem_status = query_feature_status(&deps, Features::Redeem)?;
        assert_eq!(
            redeem_status,
            "{\"status\":{\"features\":[{\"feature\":\"Redeem\",\"status\":\"NotPaused\"}]}}"
                .to_string()
        );

        let result_str = pause_feature(&mut deps, Features::Redeem, 2, "bob")?;
        assert_eq!(
            result_str,
            "{\"pause\":{\"status\":\"success\"}}".to_string()
        );

        let redeem_status = query_feature_status(&deps, Features::Redeem)?;
        assert_eq!(
            redeem_status,
            "{\"status\":{\"features\":[{\"feature\":\"Redeem\",\"status\":\"Paused\"}]}}"
                .to_string()
        );

        let error = do_redeem(&mut deps, "bob", 2, 2);
        match error {
            Err(err) => assert_eq!(
                err,
                StdError::generic_err("feature toggle: feature '\"Redeem\"' is paused")
            ),
            Ok(..) => panic!("A redeem while the feature was paused did not error"),
        }

        Ok(())
    }

    #[test]
    fn test_pause_feature_claim() -> StdResult<()> {
        let mut deps = init_helper(None, None)?;

        add_pauser(&mut deps, "bob", 3, None)?;
        let claim_status = query_feature_status(&deps, Features::Claim)?;
        assert_eq!(
            claim_status,
            "{\"status\":{\"features\":[{\"feature\":\"Claim\",\"status\":\"NotPaused\"}]}}"
                .to_string()
        );

        let result_str = pause_feature(&mut deps, Features::Claim, 2, "bob")?;
        assert_eq!(
            result_str,
            "{\"pause\":{\"status\":\"success\"}}".to_string()
        );

        let claim_status = query_feature_status(&deps, Features::Claim)?;
        assert_eq!(
            claim_status,
            "{\"status\":{\"features\":[{\"feature\":\"Claim\",\"status\":\"Paused\"}]}}"
                .to_string()
        );

        let error = do_claim(&mut deps, "bob", 2);
        match error {
            Err(err) => assert_eq!(
                err,
                StdError::generic_err("feature toggle: feature '\"Claim\"' is paused")
            ),
            Ok(..) => panic!("A claim while the feature was paused did not error"),
        }

        Ok(())
    }

    #[test]
    fn test_pause_feature_deposit() -> StdResult<()> {
        let mut deps = init_helper(None, None)?;

        add_pauser(&mut deps, "bob", 3, None)?;
        let claim_status = query_feature_status(&deps, Features::Deposit)?;
        assert_eq!(
            claim_status,
            "{\"status\":{\"features\":[{\"feature\":\"Deposit\",\"status\":\"NotPaused\"}]}}"
                .to_string()
        );

        let result_str = pause_feature(&mut deps, Features::Deposit, 2, "bob")?;
        assert_eq!(
            result_str,
            "{\"pause\":{\"status\":\"success\"}}".to_string()
        );

        let deposit_status = query_feature_status(&deps, Features::Deposit)?;
        assert_eq!(
            deposit_status,
            "{\"status\":{\"features\":[{\"feature\":\"Deposit\",\"status\":\"Paused\"}]}}"
                .to_string()
        );

        let error = do_deposit(&mut deps, "bob", 2, 2);
        match error {
            Err(err) => assert_eq!(
                err,
                StdError::generic_err("feature toggle: feature '\"Deposit\"' is paused")
            ),
            Ok(..) => panic!("A deposiss while the feature was paused did not error"),
        }

        Ok(())
    }

    #[test]
    fn test_pause_feature_send_from_platform() -> StdResult<()> {
        let mut deps = init_helper(None, None)?;

        add_pauser(&mut deps, "bob", 3, None)?;
        let claim_status = query_feature_status(&deps, Features::SendFromPlatform)?;
        assert_eq!(
            claim_status,
            "{\"status\":{\"features\":[{\"feature\":\"SendFromPlatform\",\"status\":\"NotPaused\"}]}}"
                .to_string()
        );

        let result_str = pause_feature(&mut deps, Features::SendFromPlatform, 2, "bob")?;
        assert_eq!(
            result_str,
            "{\"pause\":{\"status\":\"success\"}}".to_string()
        );

        let deposit_status = query_feature_status(&deps, Features::SendFromPlatform)?;
        assert_eq!(
            deposit_status,
            "{\"status\":{\"features\":[{\"feature\":\"SendFromPlatform\",\"status\":\"Paused\"}]}}"
                .to_string()
        );

        let error = do_send_from_platform(&mut deps, "bob", HumanAddr::default(), None);
        match error {
            Err(err) => assert_eq!(
                err,
                StdError::generic_err("feature toggle: feature '\"SendFromPlatform\"' is paused")
            ),
            Ok(..) => {
                panic!("Sending from platform did not error, even though the feature was paused")
            }
        }

        Ok(())
    }
}
