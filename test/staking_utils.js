const { handleTx } = require("./utils");
const assert = require("assert");

// handles
async function lockToken(
    client,
    nftContract,
    nftHash,
    tokenId,
    waitForCommit = true,
) {
    return handleTx(
        client.tx.compute.executeContract,
        {
            sender: client.address,
            contractAddress: nftContract,
            codeHash: nftHash,
            msg: {
                lock_nft: {
                    token_id: tokenId,
                },
            },
        },
        {
            gasLimit: 500_000,
            waitForCommit,
        },
        "lockToken",
    );
}

async function unlockToken(
    client,
    nftContract,
    nftHash,
    tokenId,
    waitForCommit = true,
) {
    return handleTx(
        client.tx.compute.executeContract,
        {
            sender: client.address,
            contractAddress: nftContract,
            codeHash: nftHash,
            msg: {
                unlock_nft: {
                    token_id: tokenId,
                },
            },
        },
        {
            gasLimit: 500_000,
            waitForCommit,
        },
        "lockToken",
    );
}


// handles
const withdrawFromStaking = async (
    client,
    stakingContract,
    stakingHash,
    amountToWithdraw,
) => {
    return handleTx(
        client.tx.compute.executeContract,
        {
            sender: client.address,
            contractAddress: stakingContract,
            codeHash: stakingHash,
            msg: {
                withdraw: {
                    contract_addr: stakingContract,
                    amount: amountToWithdraw.toString(),
                },
            },
        },
        {
            gasLimit: 500_000,
            waitForCommit: false,
        },
        "withdrawFromStaking",
    );
}

const addStakingContractAsSubscriberToNftLock = async (
    client,
    nftContract,
    nftHash,
    stakingContract,
    stakingHash,
) => {
    return handleTx(
        client.tx.compute.executeContract,
        {
            sender: client.address,
            contractAddress: nftContract,
            codeHash: nftHash,
            msg: {
                add_subs: {
                    contracts: [{
                        address: stakingContract,
                        hash: stakingHash,
                    }]
                }
            },
        },
        {
            gasLimit: 500_000,
        },
        "addStakingContractAsSubscriberToNftLock",
    );
}

const addMultiplierContracts = async (
    client,
    stakingContract,
    stakingHash,
    multiplierContract,
) => {
    return handleTx(
        client.tx.compute.executeContract,
        {
            sender: client.address,
            contractAddress: stakingContract,
            codeHash: stakingHash,
            msg: {
                add_multiplier_contracts: {
                    contracts: [multiplierContract]
                },
            },
        },
        {
            gasLimit: 500_000,
        },
        "addMultiplierContracts",
    );
}

// queries
const queryStakingBalance = async (
    clientAccount,
    stakingContract,
    stakingHash,
    viewingKey,
) => {
    return clientAccount.query.compute.queryContract({
        contractAddress: stakingContract,
        codeHash: stakingHash,
        query: {
            balance: {
                address: clientAccount.address,
                key: viewingKey,
            }
        }
    });
}

const queryStakingRewards = async (
    clientAccount,
    stakingContract,
    stakingHash,
    viewingKey,
    height,
) => {
    return clientAccount.query.compute.queryContract({
        contractAddress: stakingContract,
        codeHash: stakingHash,
        query: {
            rewards: {
                address: clientAccount.address,
                key: viewingKey,
                height,
            },
        }
    });
}

const queryIsTokenLocked = async (
    clientAccount,
    nftContract,
    nftHash,
    tokenId,
    viewingKey,
) => {
    return clientAccount.query.compute.queryContract({
        contractAddress: nftContract,
        codeHash: nftHash,
        query: {
            is_locked: {
                token_id: tokenId,
                viewing_key: viewingKey,
            },
        }
    });
}

const queryBoosterItems = async (
    clientAccount,
    stakingContract,
    stakingHash,
    viewingKey,
) => {
    return clientAccount.query.compute.queryContract({
        contractAddress: stakingContract,
        codeHash: stakingHash,
        query: {
            booster_items: {
                address: clientAccount.address,
                key: viewingKey,
                page_size: 5,
            }
        }
    });
}

// utils
async function requireNotOwned(txPromise) {
    try {
        await txPromise;
        assert.fail("tx should raise an error on user now owner");
    } catch (e) {
        assert(e.tx.jsonLog.generic_err.msg.startsWith("You do not own token"));
        console.log("Correct error message on unowned token");
    }
}

async function requireLocked(txPromise) {
    const result = await txPromise;
    assert(result.is_locked.token_is_locked);
    console.log("token was indeed locked");
}

async function requireUnlocked(txPromise) {
    const result = await txPromise;
    assert(!result.is_locked.token_is_locked);
    console.log("token was indeed unlocked");
}

async function requireLockedForTransfer(txPromise) {
    try {
        await txPromise;
        assert.fail("tx should raise an error on user not allowed to transfer locked token");
    } catch (e) {
        assert(e.tx.jsonLog.generic_err.msg.startsWith("Attempting to transfer a locked token"));

    }
}

const assertRewards = (actual, expected, name) => {
    console.log(`${name}'s actual rewards:`, actual);
    console.log(`${name}'s expected rewards:`, expected);

    assert(
        actual.rewards.rewards === expected.toString() ||
        // could be 1 leftover from previous withdraws,
        // depending on the amount blocks it took in-between
        actual.rewards.rewards === (expected + 1).toString()
    );
}


module.exports = {
    lockToken,
    unlockToken,
    withdrawFromStaking,
    addStakingContractAsSubscriberToNftLock,
    addMultiplierContracts,
    queryStakingBalance,
    queryStakingRewards,
    queryIsTokenLocked,
    queryBoosterItems,
    requireNotOwned,
    requireLocked,
    requireUnlocked,
    requireLockedForTransfer,
    assertRewards,
};