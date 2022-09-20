const { createCli, storeCode, Instantiate } = require("../test/cli");
const {
    addMinters,
    setTokenAttributes,
    setContractStatus,
    mintAdminNfts,
    getNftDossier,
    addReceivingContracts, changeWhitelistLevel,
} = require("../test/utils");

const fs = require("fs");

const { resolve } = require("path");
require("dotenv").config();

const PATHS = {
    minter: "artifacts/contracts/minter_contract.wasm",
    snip20: "artifacts/contracts/snip20.wasm",
    nft: "artifacts/contracts/snip721.wasm",
};

const NFT_COUNT = Number(process.env.NFT_COUNT || "0") || 4000;

(async () => {
    const secretNetwork = await createCli(
        process.env.MNEMONIC,
        process.env.NODE_ENDPOINT,
        process.env.CHAIN_ID,
    );

    console.log(`Deployer address: ${secretNetwork.address}`);

    let tokenAddr;
    let tokenHash;
    let tokenCode;
    if (process.env.CHAIN_ID !== "secret-4") {
        [tokenCode, tokenHash] = await storeCode(PATHS.snip20, secretNetwork);

        const tokenInitMsg = {
            prng_seed: "YWE",
            symbol: "LGND",
            name: "legend",
            decimals: 6,
            initial_balances: [{ address: secretNetwork.address, amount: "10000000000" }],
            config: {
                public_total_supply: true,
                enable_deposit: true,
                enable_redeem: true,
                enable_mint: true,
                enable_burn: true,
            },
            supported_denoms: ["uscrt", "ibc/AAAAAAAAAAAAA"],
        };

        tokenAddr = await Instantiate(secretNetwork, tokenInitMsg, tokenCode, tokenHash);
    } else {
        tokenAddr = process.env.TOKEN_ADDRESS;
        tokenHash = await secretNetwork.query.compute.contractCodeHash(tokenAddr);
    }

    const [mintContractCode, mintContractHash] = await storeCode(PATHS.minter, secretNetwork);
    const [gpNftCode, gpNftHash] = await storeCode(PATHS.nft, secretNetwork);

    const royalty_info = require(resolve(process.env.ROYALTY_INFO_PATH));
    const nftInitMsg = {
        name: process.env.COLLECTION_NAME,
        entropy: process.env.ENTROPY,
        symbol: process.env.COLLECTION_SYMBOL,
        royalty_info: royalty_info,
    };

    console.log("Deploying NFT");
    const nftContractAddress = await Instantiate(
        secretNetwork,
        nftInitMsg,
        gpNftCode,
        gpNftHash,
        //process.env.NFT_LABEL,
    );

    if (!nftContractAddress) {
        console.log(`Failed to deploy nft`);
        return;
    }

    const nft_price = require(resolve(process.env.NFT_PRICE_PATH));

    if (process.env.CHAIN_ID !== "secret-4") {
        nft_price.push(  {
            "token": {
                "snip20": {
                    "address": tokenAddr,
                    "hash": tokenHash
                }
            },
            "price": "100",
            "whitelist_price": "75"
        })
    }

    const mintingContractInitMsg = {
        nft_count: NFT_COUNT,
        nft_contract: { address: nftContractAddress, hash: gpNftHash },
        price: nft_price,
        random_seed: process.env.ENTROPY,
    };
    console.log("Deploying Minter");
    const mintingContractAddress = await Instantiate(
        secretNetwork,
        mintingContractInitMsg,
        mintContractCode,
        mintContractHash,
        //process.env.MINTER_LABEL,
    );

    // const platformContractHash = await secretNetwork.query.compute.contractCodeHash(
    //     process.env.PLATFORM_ADDRESS,
    // );
    // await addReceivingContracts(
    //     secretNetwork,
    //     process.env.PLATFORM_ADDRESS,
    //     [mintingContractAddress],
    //     platformContractHash,
    // );

    await addMinters(secretNetwork, nftContractAddress, [mintingContractAddress], gpNftHash);

    const attributes = [];
    const attrFiles = fs.readdirSync(process.env.ATTRIBUTES_PATH);
    // const fileNamesMap = JSON.parse(fs.readFileSync(process.env.FILE_NAMES_PATH));

    for (let i = 0; i < NFT_COUNT; i++) {
        const metadata = JSON.parse(
            fs.readFileSync(process.env.ATTRIBUTES_PATH + "/" + attrFiles[i]),
        );

        attributes.push({
            token_id: metadata.id,
            attributes: {
                public_attributes: {
                    custom_traits: metadata.attributes,
                    token_uri: "ipfs://QmchD3sJwdhngudS1UuPk1KkH27yyq7vdLpQpzLBuL25RM/0a2e4e668364d03b9ee8110e6fe6b33beacec2ac6d33e56f18cb01af6380ad5a",
                    // process.env.BASE_URL_LOWRES + "/" + metadata.id + ".png",
                    description: metadata.description,
                    name: metadata.name,
                    external_url: metadata.external_url
                },
                private_attributes: {
                    custom_traits: metadata.attributes,
                    token_uri: "ipfs://QmbCj9iT75hgvEP7gArcvAHudL49nk3Sm5ioc9mkKozQ7Z/79.png",
                        // process.env.BASE_URL_HIGHRES + "/" + fileNamesMap[metadata.id + ".png"],
                    description: metadata.description,
                    name: metadata.name,
                    external_url: metadata.external_url,
                    media: metadata.media
                },
            },
        });
    }

    // split this into 15 txs
    const batches_num = parseInt(process.env.SET_ATTRIBUTES_BATCHES);
    for (let i = 0; i < batches_num; i++) {
        let start = (i * NFT_COUNT) / batches_num;
        let end = ((i + 1) * NFT_COUNT) / batches_num;
        await setTokenAttributes(
            secretNetwork,
            mintingContractAddress,
            attributes.slice(start, end),
            mintContractHash,
        );
        console.log(`Finished setAttrs round ${i + 1} from ${start} to ${end}`);
    }


    await changeWhitelistLevel(secretNetwork, mintingContractAddress, 3);

    // await setContractStatus(
    //     secretNetwork,
    //     nftContractAddress,
    //     "stop_transactions", // stopTransactions
    //     gpNftHash,
    // );

    console.log(`Done deployment!`);
    console.log(`NFT address: ${nftContractAddress}`);
    console.log(`Minting address: ${mintingContractAddress}`);
    console.log(`Token address: ${tokenAddr}`);
    console.log(`deployer address: ${secretNetwork.address}`);
})();
