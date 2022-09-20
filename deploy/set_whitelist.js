const { createCli } = require("../test/cli");
const { addToWhitelist } = require("../test/utils");
fs = require("fs");

require("dotenv").config();

const mintingContractAddress = process.env.MINTING_CONTRACT_ADDRESS;

(async () => {
    const secretNetwork = await createCli(
        process.env.MNEMONIC,
        process.env.NODE_ENDPOINT,
        process.env.CHAIN_ID,
    );

    // Get the contract hash of the minting contract
    const mintingContractHash =
        await secretNetwork.query.compute.contractCodeHash(
            mintingContractAddress,
        );

    // Read the whitelist file. File name is defined in the .env file.
    const whitelist = JSON.parse(
        fs.readFileSync(process.env.WHITELIST_FILE_PATH, "utf8"),
    );

    // Add all addresses to whitelist by batches
    const batches = 5;
    for (let i = 0; i < batches; i++) {
        let start = (i * whitelist.length) / batches;
        let end = ((i + 1) * whitelist.length) / batches;

        await addToWhitelist(
            secretNetwork,
            mintingContractAddress,
            whitelist.slice(start, end),
            mintingContractHash,
        );
    }
})();
