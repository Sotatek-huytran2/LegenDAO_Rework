
const { Wallet, getMsgDecoderRegistry, MsgExecuteContract, MsgSnip20Send, MsgSnip20Transfer, SecretNetworkClient } = require("secretjs");
const { ethers } = require('ethers');

const main = async () => {
    // const grpcWebUrl = " https://secret-4.api.trivium.network:1317";
    // To create a readonly secret.js client, just pass in a gRPC-web endpoint
    // const secretjs = new CosmWasmClient(grpcWebUrl, undefined, BroadcastMode.Sync);

    const wallet = new Wallet(
        "salad holiday elevator exile marble casual job extend sail wedding feed language electric gloom orphan night input oval differ mango shock year cake saddle",
    );

    const owner = new Wallet(
        "joy clip vital cigar snap column control cattle ocean scout world rude labor gun find drift gaze nurse canal soldier amazing wealth valid runway"
    );
    const myAddress = wallet.address;

    const secretjs = await SecretNetworkClient.create({
        grpcWebUrl: "https://grpc.mainnet.secretsaturn.net",
        chainId: "secret-4",
        wallet: wallet,
        walletAddress: myAddress,
    });

    // const singer = new Signer(

    // )

    // const {
    //     balance: { amount },
    // } = await secretjs.query.bank.balance(
    //     {
    //         address: "secret1pt7vpkzpm7f6n6nvcvx096gfnln4qkawhpfk8g",
    //         denom: "uscrt",
    //     } 
    // );

    // const codeHash = await secretjs.query.compute.contractCodeHash("secret18y59n8z3frrslek52tkkq6t9yk76cdp57wztn9");
    // const platformCodeHash = await secretjs.query.compute.contractCodeHash("secret1xcdxl3pfv9n3kycfhxgtul27awjw3hd4k96mrg");
    // const stakingCodeHash = await secretjs.query.compute.contractCodeHash("secret1lnk3x8q06nckmprptchh238gshzskkvjht0c8f");


    const msg_deposit = {
        deposit: {
            to: wallet.address
        }
    }

    const platform_send_msg = new MsgExecuteContract({
        contractAddress: "secret18y59n8z3frrslek52tkkq6t9yk76cdp57wztn9",
        msg: {
            send: {
                amount: "10000000",
                recipient: "secret1xcdxl3pfv9n3kycfhxgtul27awjw3hd4k96mrg",
                msg: Buffer.from(JSON.stringify(msg_deposit)).toString("base64"),
                recipient_code_hash: "2ae84f76f2411405ee17430446a0bb2754c1b70b91cea3820e37962a090374ab",
                memo: "nothing is possible",
                padding: undefined
            }
        },
        codeHash,
        sender: myAddress,
        sentFunds: []
    })

    // contract_addr: HumanAddr,
    //     /// If not specified, use all funds
    //     amount: Option<Uint128>,
    //     /// Probably not necessary
    //     memo: Option<String>,
    //     /// Wanted message to initiate at the destination contract (defined in the destination contract)
    //     msg: Binary,
    const send_from_platform_msg = new MsgExecuteContract({
        contractAddress: "secret1xcdxl3pfv9n3kycfhxgtul27awjw3hd4k96mrg",
        msg: {
            send_from_platform: {
                amount: "10000000",
                contract_addr: "secret1lnk3x8q06nckmprptchh238gshzskkvjht0c8f",
                msg: Buffer.from(
                    JSON.stringify(
                        {
                            Deposit: {}
                        }
                    )
                ).toString("base64"),
                memo: "Try staking",
            }
        },
        codeHash: "2ae84f76f2411405ee17430446a0bb2754c1b70b91cea3820e37962a090374ab",
        sender: myAddress,
        sentFunds: []
    })
// const msg_set_viewing_key = new MsgExecuteContract({
    //     contractAddress: "secret1xcdxl3pfv9n3kycfhxgtul27awjw3hd4k96mrg",
    //     msg: {
    //         set_viewing_key: {
    //             key: "hello"
    //         }
    //     },
    //     codeHash: "2ae84f76f2411405ee17430446a0bb2754c1b70b91cea3820e37962a090374ab",
    //     sender: myAddress,
    //     sentFunds: []
    // })

    const msg_set_viewing_key_for_staking = new MsgExecuteContract({
        contractAddress: "secret1lnk3x8q06nckmprptchh238gshzskkvjht0c8f",
        msg: {
            set_viewing_key: {
                key: "hello"
            }
        },
        codeHash: stakingCodeHash,
        sender: myAddress,
        sentFunds: []
    })

    const add_receiving_msg = new MsgExecuteContract({
        contractAddress: "secret1xcdxl3pfv9n3kycfhxgtul27awjw3hd4k96mrg",
        msg: {
            add_receiving_contracts: {
                addresses: [
                    "secret1lnk3x8q06nckmprptchh238gshzskkvjht0c8f"
                ]
            }
        },
        codeHash: platformCodeHash,
        sender: myAddress,
        sentFunds: []
    });

    const withdraw_msg = new MsgExecuteContract({
        contractAddress: "secret1lnk3x8q06nckmprptchh238gshzskkvjht0c8f",
        msg: {
            withdraw: {
                amount: "100"
            }
        },
        codeHash: stakingCodeHash,
        sender: myAddress,
        sentFunds: []
    });


    const msg = new MsgSnip20Transfer({
        contractAddress: "secret18y59n8z3frrslek52tkkq6t9yk76cdp57wztn9",
        msg: {
            transfer: {
                amount: "1",
                recipient: myAddress,
                padding: undefined
            }
        },
        codeHash,
        sender: myAddress,
    });

    // const tx = await secretjs.tx.broadcast([withdraw_msg], {
    //     gasLimit: 800000
    //     // gasPriceInFeeDenom: 0.000625
    // });

    // console.log(tx);

    // const config = await secretjs.query.compute.queryContract({contractAddress: "secret1xcdxl3pfv9n3kycfhxgtul27awjw3hd4k96mrg", codeHash: platformCodeHash, query: {
    //     "config": {}
    // }})

    // console.log("config: ", config);
