const { expect, use } = require("chai");
const { Contract, getAccountByName, polarChai } = require("secret-polar");
const path = require('path');
const fs = require('fs');
const sha256 = require('crypto-js/sha256');
const { BigNumber } = require('bignumber.js');
const { send } = require("process");

use(polarChai);

const snip_label = "Init snip 5"
const platform_label = "Init platform 5"
const staking_label = "Init staking 5"
const snip721_label = "Init snip721 5"
const minting_label = "Init minting 5"

const PLATFORM_VK_LGND = "PLATFORM_VK"
const STAKING_VK_LGND = "STAKING_VK"
const OWNER_VK_LGND = "OWNER_VK"


const USER_1_VK_LGND = "USER_1_VK"
const USER_1_VK_ON_PLATFORM = "USER_1_VK_PLATFORM"
const USER_1_VK_ON_STAKING = "USER_1_VK_STAKING"
const USER_1_VK_SNIP_721 = "USER_1_VK_SNIP_721"

const AMOUNT_STAKE = new BigNumber(1).multipliedBy(new BigNumber(10).pow(6));

describe("Minting", () => {

    async function setup() {

        //console.log(AMOUNT_STAKE.toFixed());

        const contract_owner = getAccountByName("account_0");
        const user_1 = getAccountByName("account_1");
        const user_2 = getAccountByName("account_4");

        const snip20_token = new Contract("snip20");
        const platform = new Contract("platform");
        const snip721_token = new Contract("snip721");
        const nft_minting = new Contract("minter-contract");


        const deploy_response_snip20 = await snip20_token.deploy(contract_owner, {
            amount: [{ amount: "750000", denom: "uscrt" }],
            gas: "3000000",
        });

        const deploy_response_platform = await platform.deploy(contract_owner, {
            amount: [{ amount: "750000", denom: "uscrt" }],
            gas: "3000000",
        });


        const deploy_response_snip721 = await snip721_token.deploy(contract_owner, {
            amount: [{ amount: "750000", denom: "uscrt" }],
            gas: "4000000",
        });


        const deploy_response_minting = await nft_minting.deploy(contract_owner, {
            amount: [{ amount: "750000", denom: "uscrt" }],
            gas: "6000000",
        });


        const snip20_code_hash = deploy_response_snip20.contractCodeHash;
        const platform_code_hash = deploy_response_platform.contractCodeHash;
        const nft_minting_code_hash = deploy_response_minting.contractCodeHash;
        const snip721_code_hash = deploy_response_snip721.contractCodeHash;



        const lgndInitMsg = {
            prng_seed: "YWE",
            symbol: "LGND",
            name: "legend",
            decimals: 6,
            initial_balances: [
                { address: contract_owner.account.address, amount: "1000000" },
                { address: user_1.account.address, amount: "1000000" },
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




        // SNIP 721
        // ==================================================================================


        const snip721InitMsg = {
            name: "LegenDAO NFT",
            entropy: "LEGENDAO_ENTROPY",
            symbol: "NFT"
        }


        await snip721_token.instantiate(
            snip721InitMsg,
            snip721_label,
            contract_owner
        );



        // // NFT MINTING
        // // ==================================================================================

        // Buffer.from(JSON.stringify("ABC")).toString("base64"),

        const nftMintingInitMsg = {
            nft_count: 400,
            nft_contract:
            {
                address: snip721_token.contractAddress,
                hash: deploy_response_snip721.contractCodeHash
            },
            base_uri: "https://concak.com/",
            random_seed: Buffer.from(JSON.stringify("ABC")).toString("base64"),
            price: [
                {
                    price: "100000000",
                    whitelist_price: "100000000",
                    token:
                    {
                        snip20:
                        {
                            address: snip20_token.contractAddress,
                            hash: deploy_response_snip20.contractCodeHash
                        }
                    },
                }
            ],
            only_platform: true
        }


        await nft_minting.instantiate(
            nftMintingInitMsg,
            minting_label,
            contract_owner,
            null,
            { // custom fees
                amount: [{ amount: "750000", denom: "uscrt" }],
                gas: "6000000",
            }
        );


        // add receive contract minting
        await platform.executeMsg(
            "add_receiving_contracts",
            {
                "addresses": [
                    nft_minting.contractAddress
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

        await snip721_token.executeMsg(
            "set_viewing_key",
            {
                "key": USER_1_VK_SNIP_721,
                "padding": null
            },
            user_1,
            undefined,
            { // custom fees
                amount: [{ amount: "750000", denom: "uscrt" }],
                gas: "3000000",
            }
        );


        return { snip20_token, platform, snip721_token, nft_minting, contract_owner, user_1, user_2, platform_code_hash, nft_minting_code_hash };
    }

    describe("Deploy", async function () {

        it("Should deposit successful", async () => {
            const { snip20_token, platform, snip721_token, nft_minting, contract_owner, user_1, user_2, platform_code_hash, nft_minting_code_hash } = await setup();

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

            // let platformBalance = await snip20_token.queryMsg(
            //     "balance",
            //     {
            //         "address": platform.contractAddress,
            //         "key": PLATFORM_VK_LGND,
            //     }
            // )

            let userOneBalanceInPlatform = await platform.queryMsg(
                "balance",
                {
                    "address": user_1.account.address,
                    "key": USER_1_VK_ON_PLATFORM,
                }
            )


            // console.log(`=============================================== Balance of Platform after first deposit: ${platformBalance.balance.amount}`);
            console.log(`=============================================== Balance of User on platform after first deposit: ${userOneBalanceInPlatform.balance.staked}`);


            // PLATFORM DEPOSIT TO MINTING ===========================================================


            const AMOUNT_MINT = new BigNumber(1).multipliedBy(new BigNumber(10).pow(6));


            const platform_deposit_minting = {
                mint: {
                    mint_for: user_1.account.address,
                    amount_avatar_to_mint: 1,
                    amount_loot_box_to_mint: 0,
                    amount_item_to_mint: 0,
                },
            }

            await platform.executeMsg(
                "send_from_platform",
                {
                    "contract_addr": nft_minting.contractAddress,
                    "amount": AMOUNT_MINT.toFixed(),
                    "msg": Buffer.from(JSON.stringify(platform_deposit_minting)).toString("base64"),
                    //"msg": platform_deposit_minting,
                    "memo": "",
                },
                user_1,
                undefined,
                { // custom fees
                    amount: [{ amount: "750000", denom: "uscrt" }],
                    gas: "3000000",
                }
            );

            
            // await snip721_token.executeMsg(
            //     "add_minters",
            //     {
            //         "minters": [contract_owner.account.address, nft_minting.contractAddress],
            //         "padding": null
            //     },
            //     contract_owner,
            //     undefined,
            //     { // custom fees
            //         amount: [{ amount: "750000", denom: "uscrt" }],
            //         gas: "3000000",
            //     }
            // );

            // await nft_minting.executeMsg(
            //     "mint_admin",
            //     {
            //         "mint_for": contract_owner.account.address,
            //         "amount": 1,
            //         "amount_loot_box_to_mint": 0,
            //         "amount_item_to_mint": 0,
            //     },
            //     contract_owner,
            //     undefined,
            //     { // custom fees
            //         amount: [{ amount: "750000", denom: "uscrt" }],
            //         gas: "3000000",
            //     }
            // );

            let userOneBalanceAfterBuy = await platform.queryMsg(
                "balance",
                {
                    "address": user_1.account.address,
                    "key": USER_1_VK_ON_PLATFORM,
                }
            )

            console.log(`=============================================== Balance of User on platform after first deposit: ${userOneBalanceAfterBuy.balance.staked}`);


            let userOneNFT = await snip721_token.queryMsg(
                "nft_info",
                {
                    "token_id": "1",
                }
            );

            // let userOneNFT = await snip721_token.queryMsg(
            //     "owner_of",
            //     {
            //         "token_id": "1",
            //         "viewer": {
            //             address: user_1.account.address,
            //             viewing_key: USER_1_VK_SNIP_721
            //         },
            //         "include_expired": undefined
            //     }
            // );

            // console.log(userOneNFT)


            // let userOneNFT = await snip721_token.queryMsg(
            //     "tokens",
            //     {
            //         "owner": user_1.account.address,
            //         "viewer": user_1.account.address,
            //         "viewing_key": USER_1_VK_SNIP_721,
            //         "start_after": undefined,
            //         "limit": undefined,
            //     },

            // );

            // let userOneNFT = await snip721_token.queryMsg(
            //     "num_tokens",
            //     {
            //         "viewer": {
            //             "address": user_1.account.address,
            //             "viewing_key": USER_1_VK_SNIP_721,
            //         },
            //     },

            // );

            console.log("====================================")

            console.log(userOneNFT)


            let check = await nft_minting.queryMsg(
                "config",
                {
                    
                }
            );

            console.log(check)




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
