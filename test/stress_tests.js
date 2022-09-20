const {createCli, storeCode, Instantiate} = require("./cli");
const {
    addMinters,
    setTokenAttributes,
    changeWhitelistLevel, sleep, mintNftsWithSnip
} = require("./utils");
const {MsgExecuteContract, MsgSend} = require("secretjs");

require("dotenv").config();

const { use } = require("chai");
const { polarChai } = require("secret-polar");

use(polarChai);

const PATHS = {
    minter: "artifacts/contracts/minter_contract.wasm",
    lgnd: "artifacts/contracts/snip20.wasm",
    nft: "artifacts/contracts/snip721.wasm"
}

const TOKEN_PRICE = Number(process.env.TOKEN_MINT_PRICE || "0") || 1000000;
const NFT_COUNT = Number(process.env.NFT_COUNT || "0") || 4000;

let test_suite = async () => {
    const secretNetwork = await createCli(process.env.MNEMONIC, process.env.NODE_ENDPOINT, process.env.CHAIN_ID);
    let account = (await secretNetwork.query.auth.account({address: secretNetwork.address})).account;

    console.log(`minting ${NFT_COUNT} tokens\nminting address: ${process.env.MINTING_CONTRACT_ADDRESS}\n$lgnd: ${process.env.LGND_ADDRESS}`);

    for (let i = 0; i < NFT_COUNT * 1.2; i++) {
        let rand = getRandomInt(5);
        if (rand === 3) { // arbitrary
            console.log("sleeping..")
            await sleep(6000);
        }
        mintNftCustomSigner(secretNetwork, process.env.MINTING_CONTRACT_ADDRESS, process.env.LGND_ADDRESS, TOKEN_PRICE, rand, {
            accountNumber: account.accountNumber,
            sequence: account.sequence,
            chainId: process.env.CHAIN_ID
        }).then((tx) => {
            if (tx.code !== 0) {
                console.log(`Error: ${tx.rawLog}`);
            } else {
                console.log(`Minted ${rand} tokens`);
                console.log(`${i / (NFT_COUNT * 1.2) * 100}%`);
                console.log(`${JSON.stringify(tx.jsonLog)}`);
                console.log(`${String.fromCharCode(...tx.data[0])}`);
            }
        }).catch((err) => {
            console.log(`Error: ${err}`);
        });
        await sleep(300);
        account.sequence = (Number(account.sequence) + 1).toString();
    }
};

describe('stress tests', function() {
    this._timeout = 1000000000;

    return; // TODO remove when this test is ready to run in CI
    it("handles stress", test_suite);
});


function getRandomInt(max) {
    return Math.floor(Math.random() * (max - 1)) + 1;
}

const mintNftCustomSigner = async (
    secretNetwork,
    nftContract,
    snipContract,
    priceForEach,
    amountToBuy,
    signerData,
    buyFor = null) => {
    let inner_msg = Buffer.from(
        JSON.stringify({
            mint: {
                amount_to_mint: amountToBuy,
                mint_for: buyFor || secretNetwork.address,
            },
        }),
    ).toString("base64");

    const message = new MsgExecuteContract({
        sender: secretNetwork.address,
        contractAddress: snipContract,
        codeHash: "485f11a6cea052c097c506ed5e1a54466fcdbc03d94486edd05b8e080ad66b18",
        // codeHash: "62c567c47a058e527705e120b7ea87e10639b0f5bf3a016f0b201f1b5390efdc",
        msg: {
            send: {
                recipient: nftContract,
                amount: (priceForEach * amountToBuy).toString(),
                msg: inner_msg,
            },
        },
    });

    let rand = getRandomInt(5);
    let messages = new Array(rand).fill(message);

    return await secretNetwork.tx.broadcast(messages, {
        explicitSignerData: signerData,
        gasLimit: 1_000_000
    });
}
