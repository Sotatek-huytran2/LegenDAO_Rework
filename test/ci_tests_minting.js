const {TxResultCode} = require("secretjs");
const assert = require("assert");
const polarConfig = require("../polar.config");
const {createAccount, fillUpFromFaucet, createCli, storeCode, Instantiate} = require("./cli");
const {
    addMinters,
    setTokenAttributes,
    mintAdminNfts,
    viewTokens,
    mintNftsWithSnip,
    mintNfts,
    changeWhitelistLevel,
    isWhitelisted,
    addToWhitelist,
    transferSnip20,
} = require("./utils");

require("dotenv").config();

const {use} = require("chai");
const {polarChai} = require("secret-polar");

use(polarChai);

console.warn = () => {
};

const PATHS = {
    minter: "artifacts/contracts/minter_contract.wasm",
    lgnd: "artifacts/contracts/snip20.wasm",
    nft: "artifacts/contracts/snip721.wasm",
};

let test_suite = async () => {
    let REST_ENDPOINT = polarConfig.networks.ci.endpoint;
    let chainId = polarConfig.networks.ci.chainId;

    const secretNetwork = await createCli(polarConfig.accounts[0].mnemonic, REST_ENDPOINT, chainId);

    // const chainId = await secretNetwork.getChainId();
    // console.log("chainId: ", chainId);
    //
    // const height = await secretNetwork.getHeight();
    // console.log("height: ", height);

    if (chainId === "secretdev-1") {
        await fillUpFromFaucet(secretNetwork, 10_000_000);
    }

    const [mintContractCode, _] = await storeCode(PATHS.minter, secretNetwork);
    const [gpNftCode, gpNftHash] = await storeCode(PATHS.nft, secretNetwork);
    const [lgndCode, lgndContractHash] = await storeCode(PATHS.lgnd, secretNetwork);

    const nftInitMsg = {
        name: "CryptidsTest",
        entropy: "YWE",
        // revealer: secretNetwork.senderAddress,
        symbol: "CRYPT",
        royalty_info: {
            decimal_places_in_rates: 3,
            royalties: [{recipient: secretNetwork.address, rate: 50}],
        },
    };

    const nftContractAddress = await Instantiate(secretNetwork, nftInitMsg, gpNftCode);

    const lgndInitMsg = {
        prng_seed: "YWE",
        symbol: "LGND",
        name: "legend",
        decimals: 6,
        initial_balances: [{address: secretNetwork.address, amount: "10000000000"}],
        config: {
            public_total_supply: true,
            enable_deposit: true,
            enable_redeem: true,
            enable_mint: true,
            enable_burn: true,
        },
        supported_denoms: ["uscrt", "ibc/AAAAAAAAAAAAA"],
    };

    const lgndToken = await Instantiate(secretNetwork, lgndInitMsg, lgndCode);

    const TOKEN_PRICE = 1000000;
    const TOKEN_WHITELIST_PRICE = 100000;
    const nft_count = 17;
    const mintingContractInitMsg = {
        nft_count,
        nft_contract: {address: nftContractAddress, hash: gpNftHash},
        price: [
            {
                token: {
                    snip20: {address: lgndToken, hash: lgndContractHash},
                },
                price: TOKEN_PRICE.toString(),
                whitelist_price: TOKEN_WHITELIST_PRICE.toString(),
            },
        ],
        random_seed: "YWE",
    };
    const mintingContractAddress = await Instantiate(
        secretNetwork,
        mintingContractInitMsg,
        mintContractCode,
    );

    await addMinters(secretNetwork, nftContractAddress, [mintingContractAddress]);
    await addMinters(secretNetwork, lgndToken, [mintingContractAddress]);

    const VIEWING_KEY = "hello";

    const attributes = [];

    for (let i = 0; i <= nft_count; i++) {
        attributes.push({
            token_id: i.toString(10),
            attributes: {
                public_attributes: {
                    custom_traits: [{trait_type: "sword_size", value: `${i}`}],
                    rarity: 0,
                    token_uri: "https://data.whicdn.com/images/311555755/original.jpg",
                    description: "cool desc",
                    name: "cool name",
                    external_url: "https://scrtlabs.com",
                    media: [{
                        url: "link to some file",
                        authentication: {}
                    }]
                },
                private_attributes: {
                    custom_traits: [{trait_type: "shield type", value: `${i}`}],
                    rarity: 0,
                    token_uri: "https://i1.sndcdn.com/artworks-000020949523-d1u8n6-t500x500.jpg",
                    description: "cool desc",
                    name: "cool name",
                    external_url: "https://scrtlabs.com",
                    media: [{"file_type": "application", "extension": "pdf", "authentication": {"key": "1"}, "url": "https://gyld.mypinata.cloud/ipfs/Qmb5UinWY79or2btEf4tqyTYxuXaCUpH9EcFfdw3zYxAfX"}, {"file_type": "video", "extension": "mp4", "authentication": {}, "url": "https://killroymainbackend.blob.core.windows.net/video/main.m3u8"}]
                },
            },
        });
    }

    await setTokenAttributes(secretNetwork, mintingContractAddress, attributes);

    await mintAdminNfts(secretNetwork, mintingContractAddress, 2);

    // await setViewingKey(secretNetwork, nftContractAddress, secretNetwork.senderAddress, VIEWING_KEY);

    let result = await secretNetwork.tx.snip20.setViewingKey({
            sender: secretNetwork.address,
            contractAddress: nftContractAddress,
            msg: {
                set_viewing_key: {
                    key: VIEWING_KEY,
                },
            },
        },
        {
            gasLimit: 30_000,
        }
    );
    // console.log(`set viewing key: ${JSON.stringify(result)}`);

    let tokens = await viewTokens(
        secretNetwork,
        nftContractAddress,
        gpNftHash,
        secretNetwork.address,
        VIEWING_KEY,
    );
    console.log(`tokens: ${JSON.stringify(tokens)}`);
    assert(tokens.length === 2, "User should have 2 minted tokens");

    console.log(`Testing failure to mint`);
    try {
        let result = await mintNfts(secretNetwork, mintingContractAddress, 2);
        assert(
            result.code !== TxResultCode.Success,
            "succeeded to mint even though minting not enabled yet",
        );
    } catch (err) {
        // should fail
    }

    console.log(`enabling whitelisted minting`);
    await changeWhitelistLevel(secretNetwork, mintingContractAddress, 2);

    let [mnemonic, accAddress] = await createAccount();
    let userNetwork = await createCli(mnemonic, REST_ENDPOINT, chainId);
    await addToWhitelist(secretNetwork, mintingContractAddress, [accAddress]);

    result = await isWhitelisted(secretNetwork, mintingContractAddress, accAddress);
    console.log(`Is user address whitelisted? ${JSON.stringify(result)}`);

    const DEPOSIT_AMOUNT = 5_000_000;

    await secretNetwork.tx.bank.send({
        fromAddress: secretNetwork.address,
        toAddress: accAddress,
        amount: [{amount: String(DEPOSIT_AMOUNT), denom: "uscrt"}],
    });

    console.log(`\tsent 5scrt from main account to user ${accAddress} for gas`);

    await transferSnip20(secretNetwork, lgndToken, accAddress, (TOKEN_PRICE * 10).toString());
    console.log(`Sent snip20 tokens to user for mint costs`);
    // mint 2
    await mintNftsWithSnip(
        userNetwork,
        mintingContractAddress,
        lgndToken,
        TOKEN_WHITELIST_PRICE,
        2,
    );

    await userNetwork.tx.snip20.setViewingKey({
            sender: accAddress,
            contractAddress: nftContractAddress,
            codeHash: gpNftHash,
            msg: {
                set_viewing_key: {
                    key: VIEWING_KEY,
                },
            },
        },
        {
            gasLimit: 30_000,
        }
    );

    tokens = await viewTokens(userNetwork, nftContractAddress, gpNftHash, accAddress, VIEWING_KEY);
    assert(tokens.length === 2, "User should have 2 minted tokens");

    result = await isWhitelisted(secretNetwork, mintingContractAddress, accAddress);
    console.log(`Is user address whitelisted? ${JSON.stringify(result)}`);

    // mint another 1
    await mintNftsWithSnip(
        userNetwork,
        mintingContractAddress,
        lgndToken,
        TOKEN_WHITELIST_PRICE,
        1,
    );

    tokens = await viewTokens(userNetwork, nftContractAddress, gpNftHash, accAddress, VIEWING_KEY);
    assert(tokens.length === 3, "User should have 3 minted tokens");

    result = await isWhitelisted(secretNetwork, mintingContractAddress, accAddress);
    console.log(`Is user address whitelisted? ${JSON.stringify(result)}`);

    // mint another 1 - should fail
    try {
        let result = await mintNftsWithSnip(
            userNetwork,
            mintingContractAddress,
            lgndToken,
            TOKEN_WHITELIST_PRICE,
            1,
        );
        assert(
            result.code !== TxResultCode.Success,
            "succeeded to mint even though whitelisted address should have no more allocation",
        );
    } catch (err) {
        // should fail
    }

    // change whitelist level
    console.log(`changing whitelist level to public`);
    await changeWhitelistLevel(secretNetwork, mintingContractAddress, 3);
    [mnemonic, accAddress] = await createAccount();
    userNetwork = await createCli(mnemonic, REST_ENDPOINT, chainId);

    await secretNetwork.tx.bank.send({
        fromAddress: secretNetwork.address,
        toAddress: accAddress,
        amount: [{amount: String(DEPOSIT_AMOUNT), denom: "uscrt"}],
    });

    console.log(`\tsent 5scrt from main account to user for gas`);

    await transferSnip20(secretNetwork, lgndToken, accAddress, (TOKEN_PRICE * 10).toString());
    console.log(`Sent snip20 tokens to user for mint costs`);

    // public mint
    // mint 10
    await mintNftsWithSnip(userNetwork, mintingContractAddress, lgndToken, TOKEN_PRICE, 10);

    await userNetwork.tx.snip20.setViewingKey({
            sender: accAddress,
            contractAddress: nftContractAddress,
            codeHash: gpNftHash,
            msg: {
                set_viewing_key: {
                    key: VIEWING_KEY,
                },
            },
        },
        {
            gasLimit: 30_000,
        }
    );

    tokens = await viewTokens(userNetwork, nftContractAddress, gpNftHash, accAddress, VIEWING_KEY);
    assert(tokens.length === 10, "User should have 10 minted tokens");
};

describe("minting", function () {
    this._timeout = 1000000000;

    it("minting works", test_suite);
});
