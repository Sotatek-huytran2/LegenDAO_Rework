const {
    fillUpFromFaucet,
    createCli,
    storeCode,
    Instantiate,
} = require("../test/cli");

require("dotenv").config();

const PATHS = {
    snip20: "artifacts/contracts/snip20.wasm",
};

(async () => {
    const secretNetwork = await createCli(
        process.env.MNEMONIC,
        process.env.NODE_ENDPOINT,
        process.env.CHAIN_ID,
    );

    console.log(`Deployer address: ${secretNetwork.address}`);

    if (process.env.CHAIN_ID === "secretdev-1") {
        await fillUpFromFaucet(secretNetwork, 10_000_000);
    }

    const [lgndCode, lgndContractHash] = await storeCode(
        PATHS.snip20,
        secretNetwork,
    );

    const lgndInitMsg = {
        prng_seed: "YWE",
        symbol: "LGND",
        name: "legend",
        decimals: 6,
        initial_balances: [
            {address: secretNetwork.address, amount: "1000000000000000"},
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

    const lgndToken = await Instantiate(
        secretNetwork,
        lgndInitMsg,
        lgndCode,
        lgndContractHash,
    );

    console.log("Success!");
    console.log(`$LGND address: ${lgndToken}`);
    console.log(`$LGND native address: ${process.env.LGND_NATIVE}`);
})();
