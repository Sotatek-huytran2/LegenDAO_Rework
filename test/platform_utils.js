const { handleTx } = require("./utils");

const withdrawFromPlatform = async (secretNetwork, platformAddress, amount, codeHash) => {
    return handleTx(
        secretNetwork.tx.compute.executeContract,
        {
            sender: secretNetwork.address,
            contractAddress: platformAddress,
            msg: {
                redeem: {
                    amount: amount.toString()
                },
            },
            codeHash,
        },
        {
            gasLimit: 100000,
        },
        "withdrawFromPlatform",
    );
};

const depositToPlatform = async (
    secretNetwork,
    platformContract,
    snipContract,
    amount,
    buyFor = null,
) => {
    let msg = Buffer.from(
        JSON.stringify({deposit: {to: buyFor || secretNetwork.address}}),
    ).toString("base64");

    return handleTx(
        secretNetwork.tx.snip20.send,
        {
            sender: secretNetwork.address,
            contractAddress: snipContract,
            msg: {
                send: {
                    recipient: platformContract,
                    amount: amount.toString(),
                    msg,
                },
            },
        },
        {gasLimit: 500_000},
        "depositToPlatform",
    );
}

const sendToMinterFromPlatform = async (
    secretNetwork,
    platformContract,
    minterContract,
    amount,
    amountToBuy,
    buyFor = null,
) => {
    let msg = Buffer.from(
        JSON.stringify({
            mint: {
                amount_to_mint: amountToBuy,
                mint_for: buyFor || secretNetwork.address,
            },
        }),
    ).toString("base64");

    return handleTx(
        secretNetwork.tx.compute.executeContract,
        {
            sender: secretNetwork.address,
            contractAddress: platformContract,
            msg: {
                send_from_platform: {
                    contract_addr: minterContract,
                    amount: amount.toString(),
                    msg,
                },
            },
        },
        { gasLimit: 500_000 },
        "SendToMinterFromPlatform",
    );
}

const sendToStakingFromPlatform = async (
    secretNetwork,
    platformContract,
    stakingContract,
    amountToDeposit,
) => {
    let msg = Buffer.from(
        JSON.stringify({ Deposit: {} }),
    ).toString("base64");

    return handleTx(
        secretNetwork.tx.compute.executeContract,
        {
            sender: secretNetwork.address,
            contractAddress: platformContract,
            msg: {
                send_from_platform: {
                    contract_addr: stakingContract,
                    amount: amountToDeposit.toString(),
                    msg,
                },
            },
        },
        { gasLimit: 500_000 },
        "SendToStakingFromPlatform",
    );
}

const queryBalanceInPlatform = async (
    secretNetwork,
    platformContract,
    platformContractHash,
    key,
) => {
    return secretNetwork.query.compute.queryContract({
        contractAddress: platformContract,
        codeHash: platformContractHash,
        query: {
            balance: {
                address: secretNetwork.address,
                key,
            }
        }
    });
}


module.exports = {
    withdrawFromPlatform,
    depositToPlatform,
    sendToMinterFromPlatform,
    sendToStakingFromPlatform,
    queryBalanceInPlatform,
};