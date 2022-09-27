const { expect, use } = require("chai");
const { Contract, getAccountByName, polarChai } = require("secret-polar");
const path = require('path');
const fs = require('fs');
const sha256 = require('crypto-js/sha256');
const { BigNumber } = require('bignumber.js');
const { send } = require("process");

use(polarChai);

const snip_label = "Init snip 1"
const platform_label = "Init platform 2"
const staking_label = "Init staking 3"

const PLATFORM_VK_LGND = "PLATFORM_VK"
const STAKING_VK_LGND = "STAKING_VK"
const OWNER_VK_LGND = "OWNER_VK"


const USER_1_VK_LGND = "USER_1_VK"
const USER_1_VK_ON_PLATFORM = "USER_1_VK_PLATFORM"
const USER_1_VK_ON_STAKING = "USER_1_VK_STAKING"

describe("staking", () => {

    async function setup() {

        const AMOUNT_STAKE = new BigNumber(50000).multipliedBy(new BigNumber(10).pow(6));

        //console.log(AMOUNT_STAKE.toFixed());

        const contract_owner = getAccountByName("account_0");
        const user_1 = getAccountByName("account_1");
        const user_2 = getAccountByName("account_4");

        const snip20_token = new Contract("snip20");
        const platform = new Contract("platform");
        const staking = new Contract("staking");


        const deploy_response_snip20 = await snip20_token.deploy(contract_owner, {
            amount: [{ amount: "750000", denom: "uscrt" }],
            gas: "3000000",
        });

        const deploy_response_platform = await platform.deploy(contract_owner, {
            amount: [{ amount: "750000", denom: "uscrt" }],
            gas: "3000000",
        });

        const deploy_response_staking = await staking.deploy(contract_owner, {
            amount: [{ amount: "750000", denom: "uscrt" }],
            gas: "3000000",
        });


        const lgndInitMsg = {
            prng_seed: "YWE",
            symbol: "LGND",
            name: "legend",
            decimals: 6,
            initial_balances: [
                { address: contract_owner.account.address, amount: "10000000000000000" },
                { address: user_1.account.address, amount: "1000000000000" },
            ],
            config: {
                public_total_supply: true,
                enable_deposit: true,
                enable_redeem: true,
                enable_mint: true,
                enable_burn: true,
            },
            supported_denoms: [process.env.LGND_NATIVE],
        };


        const resp_snip20 = await snip20_token.instantiate(
            lgndInitMsg,
            snip_label,
            contract_owner
        );


        // ==================================================================================

        const platformInitMsg = {
            token: {
                address: resp_snip20.contractAddress,
                hash: deploy_response_snip20.contractCodeHash,
            },
            token_native_denom: process.env.LGND_NATIVE,
            viewing_key: PLATFORM_VK_LGND,
        };


        const resp_platform = await platform.instantiate(
            platformInitMsg,
            platform_label,
            contract_owner
        );


        // ==================================================================================

        const stakingInitMsg = {
            token: {
                address: resp_snip20.contractAddress,
                hash: deploy_response_snip20.contractCodeHash,
            },
            platform: {
                address: resp_platform.contractAddress,
                hash: deploy_response_platform.contractCodeHash,
            },
            inflation_schedule: [{ end_block: 10_000_000, reward_per_block: "10000" }],
            viewing_key: STAKING_VK_LGND,
            prng_seed: "IAo=",
        }


        await staking.instantiate(
            stakingInitMsg,
            staking_label,
            contract_owner
        );


        // add receive contract
        await platform.executeMsg(
            "add_receiving_contracts",
            {
                "addresses": [
                    staking.contractAddress
                ]
            },
            contract_owner,
            undefined,
            { // custom fees
                amount: [{ amount: "750000", denom: "uscrt" }],
                gas: "3000000",
            }
        );

    
        await snip20_token.executeMsg(
            "increase_allowance",
            {
              "amount": `${AMOUNT_STAKE.toFixed()}`,
              "spender": platform.contractAddress,
              "expiration": null,
              "padding": null
            },
            user_1
        );

        const platform_code_hash = deploy_response_platform.contractCodeHash;
        const staking_code_hash = deploy_response_staking.contractCodeHash;

        await staking.executeMsg(
            "set_viewing_key",
            {
                "key": USER_1_VK_ON_STAKING,
                "padding": null,
            },
            user_1,
            undefined,
            { // custom fees
                amount: [{ amount: "750000", denom: "uscrt" }],
                gas: "3000000",
            }
        );

        await snip20_token.executeMsg(
            "set_viewing_key",
            {
                "key": USER_1_VK_LGND,
                "padding": null,
            },
            user_1,
            undefined,
            { // custom fees
                amount: [{ amount: "750000", denom: "uscrt" }],
                gas: "3000000",
            }
        );

        await platform.executeMsg(
            "set_viewing_key",
            {
                "key": USER_1_VK_ON_PLATFORM,
            },
            user_1,
            undefined,
            { // custom fees
                amount: [{ amount: "750000", denom: "uscrt" }],
                gas: "3000000",
            }
        );


        return { snip20_token, platform, staking, contract_owner, user_1, user_2, AMOUNT_STAKE, staking_code_hash, platform_code_hash };
    }

    describe("Deploy", async function () {

        // it("Should deploy", async () => {
        //     const { snip20_token, platform, staking } = await setup();
        //     // expect(snip20_token.contractAddress).to.be.not.null;
        //     // expect(platform.contractAddress).to.be.not.null;
        //     // expect(staking.contractAddress).to.be.not.null;
        // });

        it("Should deposit successful", async () => {
            const { snip20_token, platform, staking, contract_owner ,user_1, user_2, AMOUNT_STAKE, staking_code_hash, platform_code_hash } = await setup();

            const msg_deposit = {
                deposit: {
                    to: user_1.account.address
                }
            }

            await snip20_token.executeMsg(
                "send",
                {
                    "recipient": platform.contractAddress,
                    "recipient_code_hash": platform_code_hash,
                    "amount": AMOUNT_STAKE.toFixed(),
                    "msg": Buffer.from(JSON.stringify(msg_deposit)).toString("base64"),
                    "memo": "",
                    "padding": null,
                },
                user_1,
                undefined,
                { // custom fees
                    amount: [{ amount: "750000", denom: "uscrt" }],
                    gas: "3000000",
                }
            );

            
  
            // let platformBalance = await snip20_token.query.balance(
            //     {
            //         "address": platform.contractAddress,
            //         "key": VIEWING_KEY,
            //     }
            // );

            let platformBalance = await snip20_token.queryMsg(
                "balance",
                {
                    "address": platform.contractAddress,
                    "key": PLATFORM_VK_LGND,
                }
            )

            let userOneBalanceAfterDeposit = await snip20_token.queryMsg(
                "balance",
                {
                    "address": user_1.account.address,
                    "key": USER_1_VK_LGND,
                }
            )
          
            let userOneBalanceInPlatform = await platform.queryMsg(
                "balance",
                {
                    "address": user_1.account.address,
                    "key": USER_1_VK_ON_PLATFORM,
                }
            );

            
            expect(platformBalance.balance.amount).to.be.equal(AMOUNT_STAKE.toFixed());
            expect(userOneBalanceAfterDeposit.balance.amount).to.be.equal("950000000000"); 


            console.log(`=============================================== Balance of Platform after first deposit: ${platformBalance.balance.amount}`);
            console.log(`=============================================== Balance of User on platform after first deposit: ${userOneBalanceInPlatform.balance.staked}`);


            // PLATFORM DEPOSIT TO STAKING ===========================================================

            const platform_deposit_stakingg = {
                Deposit: {
                    
                }
            }

            await platform.executeMsg(
                "send_from_platform",
                {
                    "contract_addr": staking.contractAddress,
                    "amount": AMOUNT_STAKE.toFixed(),
                    "msg": Buffer.from(JSON.stringify(platform_deposit_stakingg)).toString("base64"),
                    "memo": "",
                },
                user_1,
                undefined,
                { // custom fees
                    amount: [{ amount: "750000", denom: "uscrt" }],
                    gas: "3000000",
                }
            );

            let tvl_staking_1 = await staking.queryMsg(
                "total_locked",
                {
                
                }
            );

            let user_staking_1 = await staking.queryMsg(
                "balance",
                {
                    "address": user_1.account.address,
                    "key": USER_1_VK_ON_STAKING
                }
            )

            let platform_after_staking_1 = await snip20_token.queryMsg(
                "balance",
                {
                    "address": platform.contractAddress,
                    "key": PLATFORM_VK_LGND,
                }
            )

            let user_platform_after_staking_1 = await platform.queryMsg(
                "balance",
                {
                    "address": user_1.account.address,
                    "key": USER_1_VK_ON_PLATFORM,
                }
            );



            console.log(`=============================================== Balance Of User on Platform After First Deposit To Staking: ${user_platform_after_staking_1.balance.staked}`)
            console.log(`=============================================== Balance of Platform after first Deposit To Staking: ${platform_after_staking_1.balance.amount}`)

  
            console.log(`=============================================== Balance Of User on Staking After First Deposit: ${user_staking_1.balance.amount}`)
            console.log(`=============================================== Total Value Lock After First Deposit: ${tvl_staking_1.total_locked.amount}`)

            


            // PLATFORM DEPOSIT TO STAKING ===========================================================



            // DEPOSIT SECOND TIME =================================================================

            const AMOUNT_STAKE_2 = new BigNumber(1333).multipliedBy(new BigNumber(10).pow(6));


            await snip20_token.executeMsg(
                "increase_allowance",
                {
                  "amount": `${AMOUNT_STAKE_2.toFixed()}`,
                  "spender": platform.contractAddress,
                  "expiration": null,
                  "padding": null
                },
                user_1
            );


            const msg_deposit_2 = {
                deposit: {
                    to: user_1.account.address
                }
            }

            await snip20_token.executeMsg(
                "send",
                {
                    "recipient": platform.contractAddress,
                    "recipient_code_hash": platform_code_hash,
                    "amount": AMOUNT_STAKE_2.toFixed(),
                    "msg": Buffer.from(JSON.stringify(msg_deposit_2)).toString("base64"),
                    "memo": "",
                    "padding": null,
                },
                user_1,
                undefined,
                { // custom fees
                    amount: [{ amount: "750000", denom: "uscrt" }],
                    gas: "3000000",
                }
            );


            let userOneBalanceInPlatform_2 = await platform.queryMsg(
                "balance",
                {
                    "address": user_1.account.address,
                    "key": USER_1_VK_ON_PLATFORM,
                }
            );

            let platformBalance_2 = await snip20_token.queryMsg(
                "balance",
                {
                    "address": platform.contractAddress,
                    "key": PLATFORM_VK_LGND,
                }
            )

            console.log(`=============================================== Balance of Platform after second deposit: ${platformBalance_2.balance.amount}`);
            console.log(`=============================================== Balance of User on platform after second deposit: ${userOneBalanceInPlatform_2.balance.staked}`);

            
            // DEPOSIT SECOND TIME =================================================================


            // PLATFORM DEPOSIT TO STAKING SECOND TIME===========================================================


            await platform.executeMsg(
                "send_from_platform",
                {
                    "contract_addr": staking.contractAddress,
                    "amount": AMOUNT_STAKE_2.toFixed(),
                    "msg": Buffer.from(JSON.stringify(platform_deposit_stakingg)).toString("base64"),
                    "memo": "",
                },
                user_1,
                undefined,
                { // custom fees
                    amount: [{ amount: "750000", denom: "uscrt" }],
                    gas: "3000000",
                }
            );


    
            let user_staking_2 = await staking.queryMsg(
                "balance",
                {
                    "address": user_1.account.address,
                    "key": USER_1_VK_ON_STAKING
                }
            )


            let tvl_staking_2 = await staking.queryMsg(
                "total_locked",
                {
                
                }
            );

            let platform_after_staking_2 = await snip20_token.queryMsg(
                "balance",
                {
                    "address": platform.contractAddress,
                    "key": PLATFORM_VK_LGND,
                }
            )

            let user_platform_after_staking_2 = await platform.queryMsg(
                "balance",
                {
                    "address": user_1.account.address,
                    "key": USER_1_VK_ON_PLATFORM,
                }
            );



            console.log(`=============================================== Balance Of User on Platform After second Deposit To Staking: ${user_platform_after_staking_2.balance.staked}`)
            console.log(`=============================================== Balance of Platform after second Deposit To Staking: ${platform_after_staking_2.balance.amount}`)
            


           
            console.log(`=============================================== Balance Of User on Staking After Second Deposit: ${user_staking_2.balance.amount}`)
            console.log(`=============================================== Total Value Lock After Second Deposit: ${tvl_staking_2.total_locked.amount}`)

            // PLATFORM DEPOSIT TO STAKING SECOND TIME===========================================================


            // DEPOSIT THIRD TIME =================================================================

            const AMOUNT_STAKE_3 = new BigNumber(13333).multipliedBy(new BigNumber(10).pow(6));


            await snip20_token.executeMsg(
                "increase_allowance",
                {
                  "amount": `${AMOUNT_STAKE_3.toFixed()}`,
                  "spender": platform.contractAddress,
                  "expiration": null,
                  "padding": null
                },
                user_1
            );



            await snip20_token.executeMsg(
                "send",
                {
                    "recipient": platform.contractAddress,
                    "recipient_code_hash": platform_code_hash,
                    "amount": AMOUNT_STAKE_3.toFixed(),
                    "msg": Buffer.from(JSON.stringify(msg_deposit)).toString("base64"),
                    "memo": "",
                    "padding": null,
                },
                user_1,
                undefined,
                { // custom fees
                    amount: [{ amount: "750000", denom: "uscrt" }],
                    gas: "3000000",
                }
            );


            let userOneBalanceInPlatform_3 = await platform.queryMsg(
                "balance",
                {
                    "address": user_1.account.address,
                    "key": USER_1_VK_ON_PLATFORM,
                }
            );

            let platformBalance_3 = await snip20_token.queryMsg(
                "balance",
                {
                    "address": platform.contractAddress,
                    "key": PLATFORM_VK_LGND,
                }
            )

            console.log(`=============================================== Balance of Platform after third deposit: ${platformBalance_3.balance.amount}`);
            console.log(`=============================================== Balance of User on platform after third deposit: ${userOneBalanceInPlatform_3.balance.staked}`);

            
            // DEPOSIT THIRD TIME =================================================================


            // PLATFORM DEPOSIT TO STAKING THIRD TIME===========================================================


            await platform.executeMsg(
                "send_from_platform",
                {
                    "contract_addr": staking.contractAddress,
                    "amount": AMOUNT_STAKE_2.toFixed(),
                    "msg": Buffer.from(JSON.stringify(platform_deposit_stakingg)).toString("base64"),
                    "memo": "",
                },
                user_1,
                undefined,
                { // custom fees
                    amount: [{ amount: "750000", denom: "uscrt" }],
                    gas: "3000000",
                }
            );


    
            let user_staking_3 = await staking.queryMsg(
                "balance",
                {
                    "address": user_1.account.address,
                    "key": USER_1_VK_ON_STAKING
                }
            )


            let tvl_staking_3 = await staking.queryMsg(
                "total_locked",
                {
                
                }
            );

            let platform_after_staking_3 = await snip20_token.queryMsg(
                "balance",
                {
                    "address": platform.contractAddress,
                    "key": PLATFORM_VK_LGND,
                }
            )

            let user_platform_after_staking_3 = await platform.queryMsg(
                "balance",
                {
                    "address": user_1.account.address,
                    "key": USER_1_VK_ON_PLATFORM,
                }
            );



            console.log(`=============================================== Balance Of User on Platform After third Deposit To Staking: ${user_platform_after_staking_3.balance.staked}`)
            console.log(`=============================================== Balance of Platform after third Deposit To Staking: ${platform_after_staking_3.balance.amount}`)
            
                
            console.log(`=============================================== Balance Of User on Staking After Third Deposit: ${user_staking_3.balance.amount}`)
            console.log(`=============================================== Total Value Lock After Third Deposit: ${tvl_staking_3.total_locked.amount}`)


            // PLATFORM DEPOSIT TO STAKING THIRD TIME=============================
            
          
            // let userOneBalanceInPlatform = await platform.queryMsg(
            //     "balance",
            //     {
            //         "address": user_1.account.address,
            //         "key": USER_1_VK_ON_PLATFORM,
            //     }
            // );

            // expect(userOneBalanceInPlatform.balance.staked).to.be.equal(AMOUNT_STAKE.toFixed());
            // //expect(userOneBalanceInPlatform.balance.pending_redeem.claimable).to.be.equal("0");

            // let totalPlatformBalance = await platform.queryMsg(
            //     "total_balances",
            //     {

            //     }
            // );

            // expect(totalPlatformBalance.total_balances.staked).to.be.equal(AMOUNT_STAKE.toFixed());


            
        });

    })

});
