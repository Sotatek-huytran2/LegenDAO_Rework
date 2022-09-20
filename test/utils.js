const assert = require("assert");
const { TxResultCode } = require("secretjs");

const sleep = async (ms) => new Promise((r) => setTimeout(r, ms));
const TX_CHECK_MS = process.env.TX_CHECK_MS ?? 6_000;

class ComputeError extends Error {
    constructor(tx, message) {
        super(message);
        this.name = "ComputeError";
        this.tx = tx;
    }
}

async function handleTx(txFunc, txArgs, txOptions, txName) {
    txName = txName || "transaction";

    const tx = await txFunc(
        txArgs,
        {
            broadcastCheckIntervalMs: TX_CHECK_MS,
            ...txOptions,
        }
    );

    console.log(`Gas used by ${txName}: ${tx.gasUsed}`);
    if (tx.code !== TxResultCode.Success && (txOptions.waitForCommit !== false)) {
        console.log(JSON.stringify(tx.jsonLog || tx));
        throw new ComputeError(tx, `Failed to run ${txName}`);
    }
    return tx;
}

async function getScrtBalance(userCli) {
    console.log("checking SCRT balance");
    let balanceResponse = await userCli.query.bank.balance({
        address: userCli.address,
        denom: "uscrt",
    });
    return balanceResponse.balance.amount;
}

async function getCurrentBlockHeight(userCli) {
    console.log("checking SCRT balance");
    let balanceResponse = await userCli.query.block({
        address: userCli.address,
        denom: "uscrt",
    });
    return balanceResponse.balance.amount;
}

async function requireUnauthorized(txPromise) {
    try {
        await txPromise;
        assert(false, "tx should raise an error for unauthorized users");
    } catch (e) {
        assert(e.tx.jsonLog.unauthorized);
    }
}

async function requireNotAnAdmin(txPromise) {
    try {
        await txPromise;
        assert(false, "tx should raise an error for non-admins");
    } catch (e) {
        assert(e.tx.jsonLog.generic_err.msg.startsWith("not an admin:"));
    }
}

async function requirePaused(txPromise) {
    try {
        await txPromise;
        assert(false, "tx should raise an error for unauthorized users");
    } catch (e) {
        assert(e.tx.jsonLog.generic_err.msg.endsWith("is paused"));
    }
}

async function waitForHeight(client, desiredHeight) {
    let checkIntervalMs = 200;

    let currentHeight = (await client.query.tendermint.getLatestBlock()).block.header.height;
    console.log("should wait? current:", currentHeight, "desired", desiredHeight);
    while (currentHeight < desiredHeight) {
        console.log(`waiting for height {}, current height: {}`, desiredHeight, currentHeight);
        await sleep(checkIntervalMs);
        currentHeight = (await client.query.tendermint.getLatestBlock()).block.header.height;
    }
    if (currentHeight > desiredHeight) {
        throw Error("desired height is in the past");
    }
}

// handles:
async function addMinters(secretNetwork, nftContract, minters, hash) {
    return handleTx(
        secretNetwork.tx.compute.executeContract,
        {
            sender: secretNetwork.address,
            contractAddress: nftContract,
            msg: { add_minters: { minters } },
            codeHash: hash,
        },
        {
            gasLimit: 30_000,
            gasPriceInFeeDenom: 0.25,
        },
        "addMinters",
    );
}

async function getNftDossier(secretNetwork, nftContract, tokenId, vk) {
    try {
        return await secretNetwork.query.compute.queryContract({
            contractAddress: nftContract,
            query: {
                nft_dossier: {
                    token_id: tokenId,
                    viewer: {
                        address: secretNetwork.address,
                        viewing_key: vk,
                    },
                },
            },
        });
    } catch (e) {
        console.log(`Failed to fetch dossier of token. ${e}`);
    }
    return null;
}

async function mintNfts(secretNetwork, nftContract, amount) {
    return handleTx(
        secretNetwork.tx.compute.executeContract,
        {
            sender: secretNetwork.address,
            contractAddress: nftContract,
            msg: { mint: { amount } },
            sentFunds: [
                {
                    denom: "uscrt",
                    amount: String(1_000_000 * Number(amount)),
                },
            ],
        },
        {
            gasPriceInFeeDenom: 0.25,
            gasLimit: 100_000
        },
        "mintNfts",
    );
}

async function mintNftsWithSnip(
    secretNetwork,
    nftContract,
    snipContract,
    priceForEach,
    amountToBuy,
    buyFor = null,
) {
    let msg = Buffer.from(
        JSON.stringify({
            mint: {
                amount_to_mint: amountToBuy,
                mint_for: buyFor || secretNetwork.address,
            },
        }),
    ).toString("base64");

    return handleTx(
        secretNetwork.tx.snip20.send,
        {
            sender: secretNetwork.address,
            contractAddress: snipContract,
            msg: {
                send: {
                    recipient: nftContract,
                    amount: (priceForEach * amountToBuy).toString(),
                    msg,
                },
            },
        },
        {
            gasLimit: 500_000,
            gasPriceInFeeDenom: 0.25,
        },
        "mintNftsWithSnip",
    );
}

async function mintAdminNfts(secretNetwork, nftContract, amount) {
    return handleTx(
        secretNetwork.tx.compute.executeContract,
        {
            sender: secretNetwork.address,
            contractAddress: nftContract,
            msg: { mint_admin: { amount } },
        },
        {
            gasLimit: 500_000,
            gasPriceInFeeDenom: 0.25,
        },
        "mintAdminNfts",
    );
}

async function setTokenAttributes(secretNetwork, nftContract, attributes, codeHash) {
    return handleTx(
        secretNetwork.tx.compute.executeContract,
        {
            sender: secretNetwork.address,
            contractAddress: nftContract,
            msg: { set_attributes: { tokens: attributes } },
            codeHash,
        },
        {
            gasLimit: 4_000_000,
            gasPriceInFeeDenom: 0.25,
        },
        "setTokenAttributes",
    );
}

async function setContractStatus(secretNetwork, nftContract, status, codeHash) {
    return handleTx(
        secretNetwork.tx.compute.executeContract,
        {
            sender: secretNetwork.address,
            contractAddress: nftContract,
            msg: { set_contract_status: { level: status } },
            codeHash,
        },
        {},
        "setContractStatus",
    );
}

async function changeWhitelistLevel(secretNetwork, nftContract, new_level, codeHash) {
    return handleTx(
        secretNetwork.tx.compute.executeContract,
        {
            sender: secretNetwork.address,
            contractAddress: nftContract,
            msg: { changing_minting_state: { mint_state: new_level } },
            codeHash,
        },
        {
            gasLimit: 50_000,
            gasPriceInFeeDenom: 0.25,
        },
        "changeWhitelistLevel",
    );
}

async function addToWhitelist(secretNetwork, nftContract, addresses, codeHash) {
    // Map every address to be { address: address, amount: 4 }
    let whitelist = addresses.map((address) => ({
        address,
        amount: 4,
    }));

    return handleTx(
        secretNetwork.tx.compute.executeContract,
        {
            sender: secretNetwork.address,
            contractAddress: nftContract,
            msg: { add_whitelist: { addresses: whitelist } },
            codeHash,
        },
        {
            gasLimit: 500_000,
            gasPriceInFeeDenom: 0.25,
        },
        "addToWhitelist",
    );
}

async function setViewingKey(
    secretNetwork,
    snipContract,
    codeHash,
    viewingKey,
) {
    return handleTx (
        secretNetwork.tx.snip20.setViewingKey,
        {
            sender: secretNetwork.address,
            contractAddress: snipContract,
            codeHash: codeHash,
            msg: {
                set_viewing_key: {
                    key: viewingKey,
                },
            },
        },
        {
            gasLimit: 30_000,
        },
        "setViewingKey"
    );
}

async function claimFromPlatform(
    client,
    platformContract,
    codeHash,
) {
    return handleTx (
        client.tx.compute.executeContract,
        {
            sender: client.address,
            contractAddress: platformContract,
            msg: { claim_redeemed: {} },
            codeHash,
        },
        {
            gasLimit: 30_000,
        },
        "claimFromPlatform"
    );
}

async function viewTokens(secretNetwork, nftContract, codeHash, address, key) {
    try {
        let resp = await secretNetwork.query.snip721.GetOwnedTokens({
            contract: {
                codeHash,
                address: nftContract,
            },
            owner: address,
            auth: {
                viewer: {
                    address: address,
                    viewing_key: key,
                },
            },
        });

        return resp?.token_list?.tokens;
    } catch (e) {
        console.log(`Failed to viewTokens ${e}`);
    }
    return null;
}

async function isWhitelisted(secretNetwork, nftContract, address) {
    try {
        return await secretNetwork.query.compute.queryContract({
            contractAddress: nftContract,
            query: { is_whitelisted: { address } },
        });
    } catch (e) {
        console.log(`Failed to viewTokens ${e}`);
    }
    return null;
}

const transferSnip20 = async (secretNetwork, contractAddress, recipient, amount) => {
    return handleTx(
        secretNetwork.tx.snip20.transfer,
        {
            contractAddress: contractAddress,
            sender: secretNetwork.address,
            msg: {
                transfer: {
                    recipient,
                    amount,
                },
            },
        },
        { gasLimit: 200_000 },
        "transferSnip20",
    );
};

const transferSnip721 = async (client, contractAddress, contractHash, recipient, tokenId) => {
    return handleTx(
        client.tx.snip721.send,
        {
            contractAddress: contractAddress,
            sender: client.address,
            contractHash: contractHash,
            msg: {
                send_nft: {
                    contract: recipient,
                    token_id: tokenId,
                }
            },
        },
        { gasLimit: 200_000 },
        "transferSnip721",
    );
};

async function addReceivingContracts(secretNetwork, platformContract, addresses, hash) {
    return handleTx(
        secretNetwork.tx.compute.executeContract,
        {
            sender: secretNetwork.address,
            contractAddress: platformContract,
            msg: { add_receiving_contracts: { addresses } },
            codeHash: hash,
        },
        {
            gasLimit: 100_000,
        },
        "addReceivingContracts",
    );
}

const wrapNative = async (secretNetowrk, lgndAddress, amount, denom, codeHash) => {
    return handleTx(
        secretNetowrk.tx.compute.executeContract,
        {
            sender: secretNetowrk.address,
            contractAddress: lgndAddress,
            msg: {
                deposit: {},
            },
            sentFunds: [{ amount, denom }],
            codeHash,
        },
        {
            gasLimit: 100_000,
        },
        "wrapNative",
    );
};

async function addPauser(secretNetwork, platformContract, address, hash) {
    return handleTx(
        secretNetwork.tx.compute.executeContract,
        {
            sender: secretNetwork.address,
            contractAddress: platformContract,
            msg: { features: { set_pauser: { address } } },
            codeHash: hash,
        },
        {
            gasLimit: 100_000,
        },
        "addPauser",
    );
}

async function pauseFeatures(secretNetwork, platformContract, features, hash) {
    return handleTx(
        secretNetwork.tx.compute.executeContract,
        {
            sender: secretNetwork.address,
            contractAddress: platformContract,
            msg: { features: { pause: { features } } },
            codeHash: hash,
        },
        {
            gasLimit: 100_000,
        },
        "pauseFeatures",
    );
}

async function unpauseFeatures(secretNetwork, platformContract, features, hash) {
    return handleTx(
        secretNetwork.tx.compute.executeContract,
        {
            sender: secretNetwork.address,
            contractAddress: platformContract,
            msg: { features: { unpause: { features } } },
            codeHash: hash,
        },
        {
            gasLimit: 100_000,
        },
        "unpauseFeatures",
    );
}

//queries:
const queryLgndBalance = async (secretNetwork, tokenAddress, tokenCode, address, viewingKey) => {
    const balance = await secretNetwork.query.snip20.getBalance({
        contract: { address: tokenAddress, codeHash: tokenCode },
        address,
        auth: { key: viewingKey }
    });

    return parseInt(balance.balance.amount);
};

const queryIsPauser = async (secretNetwork, platformContract, address) => {
    const result = await secretNetwork.query.compute.queryContract({
        contractAddress: platformContract,
        query: {
            features: {
                is_pauser: { address },
            }
        },
    });

    return result.is_pauser.is_pauser;
};

const querySubscribers = async (client, nftContract, nftHash) => {
    return client.query.compute.queryContract({
        contractAddress: nftContract,
        contractHash: nftHash,
        query: {
            subscribers: {}
        },
    });
};

const queryStatusOfFeatures = async (secretNetwork, platformContract, features) => {
    const result = await secretNetwork.query.compute.queryContract({
        contractAddress: platformContract,
        query: {
            features: {
                status: { features }
            }
        },
    });

    return result.status.features;
};

module.exports = {
    ComputeError,
    handleTx,
    //sendSnip20,
    transferSnip20,
    transferSnip721,
    isWhitelisted,
    viewTokens,
    addToWhitelist,
    changeWhitelistLevel,
    getNftDossier,
    mintNftsWithSnip,
    mintAdminNfts,
    setTokenAttributes,
    mintNfts,
    addMinters,
    getScrtBalance,
    sleep,
    addReceivingContracts,
    setContractStatus,
    wrapNative,
    setViewingKey,
    queryLgndBalance,
    claimFromPlatform,
    queryIsPauser,
    queryStatusOfFeatures,
    addPauser,
    pauseFeatures,
    unpauseFeatures,
    requireUnauthorized,
    requireNotAnAdmin,
    requirePaused,
    querySubscribers,
    waitForHeight,
};
