
const { send } = require("process");
const { Wallet, getMsgDecoderRegistry, MsgExecuteContract, MsgSnip20Send, MsgSnip20Transfer, SecretNetworkClient } = require("secretjs");


const main = async () => {
    // const grpcWebUrl = " https://secret-4.api.trivium.network:1317";
    // To create a readonly secret.js client, just pass in a gRPC-web endpoint
    // const secretjs = new CosmWasmClient(grpcWebUrl, undefined, BroadcastMode.Sync);

    const wallet = new Wallet(
        "joy clip vital cigar snap column control cattle ocean scout world rude labor gun find drift gaze nurse canal soldier amazing wealth valid runway"
    );

    const owner = new Wallet(
        "joy clip vital cigar snap column control cattle ocean scout world rude labor gun find drift gaze nurse canal soldier amazing wealth valid runway"
    );

    const myAddress = wallet.address;

    const ownerAddress = owner.address;

    // const secretjs = await SecretNetworkClient.create({
    //     grpcWebUrl: "https://grpc.mainnet.secretsaturn.net",
    //     chainId: "secret-4",
    //     wallet: wallet,
    //     walletAddress: myAddress,
    // });

    const secretjs = await SecretNetworkClient.create({
        grpcWebUrl: "https://grpc.testnet.secretsaturn.net",
        chainId: "pulsar-2",
        wallet: wallet,
        walletAddress: myAddress,
    });

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
    const nftCodeHash = await secretjs.query.compute.contractCodeHash("secret1j5e8yyzljq9c78tjlshtvefz8fslzupfy5l6r0");

    // const msg_deposit = {
    //     deposit: {
    //         to: wallet.address
    //     }
    // }

    // const platform_send_msg = new MsgExecuteContract({
    //     contractAddress: "secret18y59n8z3frrslek52tkkq6t9yk76cdp57wztn9",
    //     msg: {
    //         send: {
    //             amount: "10000000",
    //             recipient: "secret1xcdxl3pfv9n3kycfhxgtul27awjw3hd4k96mrg",
    //             msg: Buffer.from(JSON.stringify(msg_deposit)).toString("base64"),
    //             recipient_code_hash: "2ae84f76f2411405ee17430446a0bb2754c1b70b91cea3820e37962a090374ab",
    //             memo: "nothing is possible",
    //             padding: undefined
    //         }
    //     },
    //     codeHash,
    //     sender: myAddress,
    //     sentFunds: []
    // })

    const ContractStatus = {
        ContractStatus: "stop_transactions"
    }

    const set_status_msg = new MsgExecuteContract({
        contractAddress: "secret1j5e8yyzljq9c78tjlshtvefz8fslzupfy5l6r0",
        msg: {
            set_contract_status: {
                level: "stop_transactions",
                padding: undefined
            }
        },
        codeHash: nftCodeHash,
        sender: ownerAddress,
        sentFunds: []
    })

    // const nft_mint_msg = new MsgExecuteContract({
    //     contractAddress: "secret1j5e8yyzljq9c78tjlshtvefz8fslzupfy5l6r0",
    //     msg: {
    //         mint_nft: {
                
    //         }
    //     },
    //     nftCodeHash,
    //     sender: ownerAddress,
    //     sentFunds: []
    // })

    const tx = await secretjs.tx.broadcast([set_status_msg], {
        gasLimit: 800000
        // gasPriceInFeeDenom: 0.000625
    });

    console.log(tx);

    // const config = await secretjs.query.compute.queryContract({contractAddress: "secret1xcdxl3pfv9n3kycfhxgtul27awjw3hd4k96mrg", codeHash: platformCodeHash, query: {
    //     "config": {}
    // }})

    // console.log("config: ", config);
}

main()

//module.exports = { default: run };