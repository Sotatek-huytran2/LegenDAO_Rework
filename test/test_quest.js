const { expect, use } = require("chai");
const { Contract, getAccountByName, polarChai } = require("secret-polar");

use(polarChai);

const {fillUpFromFaucet, createCli} = require("./cli.js");

const toBase64 = s => Buffer.from(s).toString("base64");
const fromBase64 = s => Buffer.from(s, "base64").toString("utf-8");

describe('Quest Tracker', function() {
    this._timeout = 1000000000;

    async function setup() {
        const contract_owner = getAccountByName("account_0");
        const other_account = getAccountByName("account_1");

        const airdrop = new Contract("airdrop");
        const snip20 = new Contract("snip20");
        const quest = new Contract("quest-tracker");
        const platform = new Contract("platform");

        await airdrop.deploy(contract_owner);
        await quest.deploy(contract_owner);
        await snip20.deploy(contract_owner);
        await platform.deploy(contract_owner);

        // await airdrop.parseSchema();
        // await snip20.parseSchema();
        // await quest.parseSchema();
        //await platform.parseSchema();

        return { contract_owner, other_account, quest, airdrop, snip20, platform };
    }

    it("Basic Test", async () => {

        const AIRDROP_AMOUNT = 1;
        const QUEST_WEIGHT = 2;
        const QUEST_NUMBER = 1;

        const { contract_owner, other_account, quest, snip20, airdrop, platform } = await setup();

        let owner_address = contract_owner.account.address;
        let other_address = other_account.account.address;

        await snip20.instantiate({
            name: "Legend Token",
            symbol: "LGND",
            decimals: 6,
            initial_balances: [{address: owner_address, amount: "1000000"}],
            prng_seed: toBase64("random seed"),
            config: {
                public_total_supply: true,
                enable_deposit: false,
                enable_redeem: false,
                enable_mint: true,
                enable_burn: true,
            }
        }, "LGND", contract_owner);

//    pub admin: Option<HumanAddr>,
//     pub platform: SecretContract,
//     pub token: SecretContract,
//     pub quest_contract: Option<HumanAddr>,
//
//         const platformInitMsg = {
//             token: {
//                 address: lgndToken,
//                 hash: lgndContractHash,
//             },
//             viewing_key: VIEWING_KEY,
//             token_native_denom: "uscrt",
//         };
        await platform.instantiate({
            token: {address: snip20.contractAddress, hash: snip20.contractCodeHash},
            token_native_denom: "uscrt",
            viewing_key: "hello",
        }, "platform", contract_owner);

        await airdrop.instantiate({
            token: {address: snip20.contractAddress, hash: snip20.contractCodeHash},
            // don't need a real platform for these tests
            platform: {address: platform.contractAddress, hash: platform.contractCodeHash},
        }, "airdrop test", contract_owner);

        await quest.instantiate({
            token: {address: snip20.contractAddress, hash: snip20.contractCodeHash},
            platform: {address: platform.contractAddress, hash: platform.contractCodeHash},
            quest_contracts: [{
                contract: owner_address,
                quest: QUEST_NUMBER,
            }],
            quest_weights: [{
                quest: QUEST_NUMBER,
                weight: QUEST_WEIGHT
            }]
        }, "quest test", contract_owner);

        console.log(`snip20 at:  ${snip20.contractAddress}`);
        console.log(`airdrop at: ${airdrop.contractAddress}`);
        console.log(`quest at:  ${quest.contractAddress}`);

        await airdrop.executeMsg("change_config", { quest_contract: quest.contractAddress },
            contract_owner);

        await quest.executeMsg("register_airdrop_contract", { contract:
                    {address: airdrop.contractAddress, hash: airdrop.contractCodeHash} },
            contract_owner);

        // create VK
        let other_vk_string = "other vk";
        await platform.executeMsg("set_viewing_key", { key: other_vk_string }, other_account);

        let assert_user_balance = async (amount) => {
            let snip20_balance_query = platform.queryMsg("balance", {address: other_address, key:other_vk_string});
            await expect(snip20_balance_query).to.respondWith({balance:{staked:amount.toString(),pending_redeem:{unbondings:[],claimable:"0"}}});
        }

        // Send SNIP-20 to the quest contract's address
        await snip20.executeMsg("transfer", { recipient: quest.contractAddress, amount: "1000" },
            contract_owner);

        // Send SNIP-20 to the airdrop contract's address
        await snip20.executeMsg("transfer", { recipient: airdrop.contractAddress, amount: "1000" },
            contract_owner);

        await airdrop.executeMsg("confirm_airdrop", {
            airdrops: [{
                address: other_address,
                to: other_address,
                amount: AIRDROP_AMOUNT.toString()
            }]
        }, contract_owner);

        await assert_user_balance(AIRDROP_AMOUNT.toString())

        await quest.executeMsg("complete_quest", {
            address: other_address
        }, contract_owner);

        // check that we received another QUEST_WEIGHT times the airdrop amount
        await assert_user_balance((AIRDROP_AMOUNT * (QUEST_WEIGHT + AIRDROP_AMOUNT)).toString())
    });
});

