const { expect, use } = require("chai");
const { Contract, getAccountByName, polarChai } = require("secret-polar");

use(polarChai);

const { fillUpFromFaucet, createCli } = require("./cli.js");

const toBase64 = (s) => Buffer.from(s).toString("base64");
const fromBase64 = (s) => Buffer.from(s, "base64").toString("utf-8");

describe("vesting", function () {
    this._timeout = 1000000000;

    async function setup() {
        const contract_owner = getAccountByName("account_0");
        const other_account = getAccountByName("account_1");

        const snip20 = new Contract("snip20");
        const vesting = new Contract("vesting");

        await snip20.deploy(contract_owner);
        await vesting.deploy(contract_owner);
        await snip20.parseSchema();

        return { contract_owner, other_account, vesting, snip20 };
    }

    it("vesting works", async () => {
        let now = Math.floor(Date.now() / 1000); // Round the milliseconds to seconds
        let next_week = now + 60 * 60 * 24 * 7;
        let next_week_x2 = now + 60 * 60 * 24 * 7 * 2;
        let last_week = now - 60 * 60 * 24 * 7;
        let last_week_x2 = now - 60 * 60 * 24 * 7 * 2;

        let schedule = {
            start_time: next_week,
            allocation: "1000",
            rate: "23", // arbitrary
            releases: [7, 7, 7, 7, 7], // a week every time
            claimed: "0", // Only used in queries
        };

        const { contract_owner, other_account, vesting, snip20 } = await setup();

        let owner_address = contract_owner.account.address;
        let other_address = other_account.account.address;

        await snip20.instantiate(
            {
                name: "Legend Token",
                symbol: "LGND",
                decimals: 6,
                initial_balances: [{ address: owner_address, amount: "1000000" }],
                prng_seed: toBase64("random seed"),
                config: {
                    public_total_supply: true,
                    enable_deposit: false,
                    enable_redeem: false,
                    enable_mint: true,
                    enable_burn: true,
                },
            },
            "LGND",
            contract_owner,
        );

        await vesting.instantiate(
            {
                vesting_token: { address: snip20.contractAddress, hash: snip20.contractCodeHash },
                vesting_token_vk: "my super secret vk",
                prng_seed: toBase64("random seed"),
                schedules: [[other_address, schedule]],
            },
            "deploy test",
            contract_owner,
        );

        console.log(`vesting at: ${vesting.contractAddress}`);
        console.log(`snip20 at:  ${snip20.contractAddress}`);

        // Set VK for both accounts
        let owner_vk_string = "owner vk";
        let other_vk_string = "other vk";
        let owner_vk = { viewing_key: { address: owner_address, key: owner_vk_string } };
        let other_vk = { viewing_key: { address: other_address, key: other_vk_string } };

        // Check that viewing keys deny access when not set
        let fund_status_msg = { query: { admin: { fund_status: {} } }, auth: owner_vk };
        let fund_status_query = vesting.queryMsg("with_auth", fund_status_msg);
        await expect(fund_status_query).to.be.revertedWith('{"unauthorized":{}}');

        let chainId = await other_account.client.getChainId();
        if (chainId === "secretdev-1") {
            let other_client = await createCli(
                other_account.account.mnemonic,
                vesting.env.network.config.endpoint,
                vesting.env.network.name,
            );
            await fillUpFromFaucet(other_client, 100_000_000);
        }

        await vesting.executeMsg("set_viewing_key", { key: owner_vk_string }, contract_owner);
        await vesting.executeMsg("set_viewing_key", { key: other_vk_string }, other_account);
        await snip20.executeMsg("set_viewing_key", { key: owner_vk_string }, contract_owner);
        await snip20.executeMsg("set_viewing_key", { key: other_vk_string }, other_account);

        // Check that that incorrect viewing keys are not allowed
        let wrong_owner_vk = { viewing_key: { address: owner_address, key: "wrong vk" } };
        let wrong_fund_status_msg = { query: { admin: { fund_status: {} } }, auth: wrong_owner_vk };
        fund_status_query = vesting.queryMsg("with_auth", wrong_fund_status_msg);
        await expect(fund_status_query).to.be.revertedWith('{"unauthorized":{}}');

        let check_fund_status = async (allocated, reserve) => {
            let fund_status_msg = { query: { admin: { fund_status: {} } }, auth: owner_vk };
            let fund_status_query = vesting.queryMsg("with_auth", fund_status_msg);
            await expect(fund_status_query).to.respondWith({ fund_status: { allocated, reserve } });
        };

        let check_admin_balance = async (amount) => {
            let snip20_balance_query = snip20.query.balance({
                address: owner_address,
                key: owner_vk_string,
            });
            await expect(snip20_balance_query).to.respondWith({ balance: { amount } });
        };

        let check_user_balance = async (amount) => {
            let snip20_balance_query = snip20.query.balance({
                address: other_address,
                key: other_vk_string,
            });
            await expect(snip20_balance_query).to.respondWith({ balance: { amount } });
        };

        // Check fund status
        await check_fund_status("1000", "0");

        // Send SNIP-20 to the vesting contract's address from the admin
        await snip20.executeMsg(
            "transfer",
            { recipient: vesting.contractAddress, amount: "1000" },
            contract_owner,
        );

        await check_admin_balance("999000");

        await check_fund_status("1000", "1000");

        // Emergency redeem to admin
        await expect(
            vesting.executeMsg("set_contract_mode", { mode: "emergency" }, other_account),
        ).to.be.revertedWith(
            `{"generic_err":{"msg":"Address ${other_address} is not allowed to perform this operation"}}`,
        );
        await expect(
            vesting.executeMsg("emergency_redeem_all", {}, contract_owner),
        ).to.be.revertedWith(
            '{"generic_err":{"msg":"Contract mode must be set to Emergency in order to withdraw all funds"}}',
        );
        await vesting.executeMsg("set_contract_mode", { mode: "emergency" }, contract_owner);
        await vesting.executeMsg("emergency_redeem_all", {}, contract_owner);

        await check_admin_balance("1000000");

        await check_fund_status("1000", "0");

        // Return contract to normal operation
        await vesting.executeMsg("set_contract_mode", { mode: "normal" }, contract_owner);

        // Send SNIP-20 to the vesting contract's address from the admin
        await snip20.executeMsg(
            "transfer",
            { recipient: vesting.contractAddress, amount: "1000" },
            contract_owner,
        );
        await check_fund_status("1000", "1000");

        let check_user_schedule = async (available) => {
            let expected_response = { balance: { address: other_address, available, schedule } };
            // Query from the user
            let balance_query = vesting.queryMsg("with_auth", {
                query: { balance: {} },
                auth: other_vk,
            });
            await expect(balance_query).to.respondWith(expected_response);
            // Query from the admin
            let balance_of_query = vesting.queryMsg("with_auth", {
                query: { admin: { balance_of: { address: other_address } } },
                auth: owner_vk,
            });
            await expect(balance_of_query).to.respondWith(expected_response);
        };

        let set_user_schedule = async () => {
            await vesting.executeMsg(
                "add_accounts",
                { accounts: [[other_address, schedule]] },
                contract_owner,
            );
        };

        let claim_user_funds = async (amount) => {
            await vesting.executeMsg("claim", { amount }, other_account);
        };

        // Check that the vesting system works as expected
        await check_user_balance("0");
        await check_user_schedule("0");
        await expect(claim_user_funds("10")).to.be.revertedWith(
            '{"generic_err":{"msg":"Not enough funds are available to withdraw yet. 10 > 0"}}',
        );

        schedule.start_time = now;
        await set_user_schedule();
        await check_user_balance("0");
        await check_user_schedule("23");
        await expect(claim_user_funds("24")).to.be.revertedWith(
            '{"generic_err":{"msg":"Not enough funds are available to withdraw yet. 24 > 23"}}',
        );

        await claim_user_funds("4");
        schedule.claimed = "4";
        await check_user_balance("4");
        await check_user_schedule("19");
        await check_fund_status("1000", "996");

        await claim_user_funds("6");
        schedule.claimed = "10";
        await check_user_balance("10");
        await check_user_schedule("13");
        await check_fund_status("1000", "990");

        schedule.start_time = last_week;
        await set_user_schedule();
        schedule.claimed = "0"; // resetting the user schedule has reset this counter
        await check_user_balance("23"); // resetting the user schedule releases all previously available funds
        await check_user_schedule("46"); // A double release since last week
        await check_fund_status("1000", "977");

        await claim_user_funds("4");
        schedule.claimed = "4";
        await check_user_balance("27");
        await check_user_schedule("42");
        await check_fund_status("1000", "973");

        await claim_user_funds("6");
        schedule.claimed = "10";
        await check_user_balance("33");
        await check_user_schedule("36");
        await check_fund_status("1000", "967");

        await claim_user_funds(null); // release all
        schedule.claimed = "46";
        await check_user_balance("69");
        await check_user_schedule("0");
        await check_fund_status("1000", "931");

        // Reset schedule
        schedule.start_time = last_week;
        await set_user_schedule();
        schedule.claimed = "0";
        await check_user_balance("69");
        await check_user_schedule("46");
        await check_fund_status("1000", "931");

        // Stop claims
        await vesting.executeMsg("set_contract_mode", { mode: "paused_claims" }, contract_owner);
        await expect(claim_user_funds("10")).to.be.revertedWith(
            '{"generic_err":{"msg":"This operation is not permitted because claims are paused"}}',
        );

        await vesting.executeMsg("set_contract_mode", { mode: "emergency" }, contract_owner);
        await expect(claim_user_funds("10")).to.be.revertedWith(
            '{"generic_err":{"msg":"This operation is not permitted because claims are paused"}}',
        );

        // Return claims
        await vesting.executeMsg("set_contract_mode", { mode: "normal" }, contract_owner);
        await claim_user_funds("10");
        schedule.claimed = "10";
        await check_user_balance("79");
        await check_user_schedule("36");
        await check_fund_status("1000", "921");
    });
});
