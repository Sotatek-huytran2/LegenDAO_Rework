const child_process = require("child_process");

const { expect, use } = require("chai");
const { Contract, getAccountByName, polarChai } = require("secret-polar");
const polarConfig = require("../polar.config");

use(polarChai);

const { fillUpFromFaucet, createCli } = require("./cli.js");
const { handleTx } = require("./utils.js");

const toBase64 = (s) => Buffer.from(s).toString("base64");
const fromBase64 = (s) => Buffer.from(s, "base64").toString("utf-8");

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function toContractObj(contract) {
    return { address: contract.contractAddress, hash: contract.contractCodeHash };
}

async function snip20Deposit(secretNetwork, snip20Address, snip20Hash, amount) {
    return handleTx(
        secretNetwork.tx.compute.executeContract,
        {
            sender: secretNetwork.address,
            contractAddress: snip20Address,
            codeHash: snip20Hash,
            msg: { deposit: {} },
            sentFunds: [{ amount, denom: "uscrt" }],
        },
        {
            gasLimit: 60_000,
            gasPriceInFeeDenom: 0.25,
        },
        "snip20Deposit",
    );
}

describe("airdrop", function () {
    this._timeout = 1000000000;

    async function setup() {
        const admin = getAccountByName("account_0");
        const user = getAccountByName("account_1");

        const lgnd = new Contract("snip20");
        const platform = new Contract("platform");
        const airdrop = new Contract("airdrop");

        await lgnd.deploy(admin);
        await platform.deploy(admin);
        await airdrop.deploy(admin);

        await lgnd.parseSchema();

        return { admin, user, lgnd, platform, airdrop };
    }

    it("airdrop works", async () => {
        // I'm keeping this file here as a reference for deploying the LGND airdrop
        return;

        let REST_ENDPOINT = polarConfig.networks.ci.endpoint;
        let chainId = polarConfig.networks.ci.chainId;

        const secretNetwork = await createCli(
            polarConfig.accounts[0].mnemonic,
            REST_ENDPOINT,
            chainId,
        );

        let { admin, user, lgnd, platform, airdrop } = await setup();

        let admin_address = admin.account.address;
        let user_address = user.account.address;

        let initial_admin_balance = "1000000";
        await lgnd.instantiate(
            {
                name: "Legend Token",
                symbol: "LGND",
                decimals: 6,
                initial_balances: [],
                prng_seed: toBase64("random seed"),
                supported_denoms: ["uscrt"],
                config: {
                    public_total_supply: true,
                    enable_deposit: true,
                    enable_redeem: true,
                    enable_mint: true,
                    enable_burn: true,
                },
            },
            "lgnd",
            admin,
        );
        await snip20Deposit(
            secretNetwork,
            lgnd.contractAddress,
            lgnd.contractCodeHash,
            initial_admin_balance,
        );

        let platform_vk = "platform-viewing-key";
        await platform.instantiate(
            {
                token: toContractObj(lgnd),
                token_native_denom: "uscrt", // Just for testing locally. should be an ibc token
                unbonding_period: 7,
                receiving_contracts: [],
                viewing_key: platform_vk,
            },
            "lgnd-platform",
            admin,
        );

        await airdrop.instantiate(
            {
                platform: toContractObj(platform),
                token: toContractObj(lgnd),
            },
            "lgnd-airdrop",
            admin,
        );

        await platform.executeMsg(
            "add_receiving_contracts",
            { addresses: [airdrop.contractAddress] },
            admin,
        );

        let admin_vk = "admin-viewing-key";
        await lgnd.executeMsg("set_viewing_key", { key: admin_vk }, admin);
        let user_vk = "user-viewing-key";
        await lgnd.executeMsg("set_viewing_key", { key: user_vk }, user);

        let balance = await lgnd.query.balance({ address: admin_address, key: admin_vk }, admin);
        console.log(JSON.stringify(balance));
        balance = await lgnd.query.balance({ address: user_address, key: user_vk }, user);
        console.log(JSON.stringify(balance));

        // let azure = child_process.spawn("func", ["host", "start"], {
        //     stdio: "inherit",
        //     cwd: "./services/airdrop",
        //     env: {
        //         AIRDROP_CONTRACT_ADDRESS: airdrop.contractAddress,
        //         AIRDROP_CONTRACT_HASH: airdrop.contractCodeHash,
        //     },
        // });

        // await sleep(5000);
        // azure.kill();
    });
});
