const {
    fillUpFromFaucet,
    createCli,
    storeCode,
    Instantiate,
} = require("../test/cli");
const {
    addReceivingContracts,
} = require("../test/utils");
require("dotenv").config();

const VIEWING_KEY = "hello";

const PATHS = {
    platform: "artifacts/contracts/platform.wasm",
    staking: "artifacts/contracts/staking.wasm",
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

    const lgndToken = process.env.LGND_ADDRESS;
    const lgndContractHash = await secretNetwork.query.compute.contractCodeHash(lgndToken);

    const [platformCode, platformContractHash] = await storeCode(
        PATHS.platform,
        secretNetwork,
    );
    const platformInitMsg = {
        token: {
            address: lgndToken,
            hash: lgndContractHash,
        },
        token_native_denom: process.env.LGND_NATIVE,
        viewing_key: VIEWING_KEY,
    };
    
    const platformContract = await Instantiate(
        secretNetwork,
        platformInitMsg,
        platformCode,
        platformContractHash,
    );

    const [stakingCode, stakingContractHash] = await storeCode(PATHS.staking, secretNetwork);
    
    const stakingInitMsg = {
        token: {
            address: lgndToken,
            hash: lgndContractHash,
        },
        platform: {
            address: platformContract,
            hash: platformContractHash,
        },
        inflation_schedule: [{end_block: 10_000_000, reward_per_block: "1"}],
        viewing_key: VIEWING_KEY,
        prng_seed: "IAo=",
    }
    const stakingContract = await Instantiate(secretNetwork, stakingInitMsg, stakingCode, stakingContractHash);

    await addReceivingContracts(secretNetwork, platformContract, [
        stakingContract,
    ], platformContractHash);

    console.log("Success!");
    console.log(`$LGND address: ${lgndToken}`);
    console.log(`Platform address: ${platformContract}`);
    console.log(`Staking address: ${stakingContract}`);
})();
