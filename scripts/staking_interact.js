const { Wallet, getMsgDecoderRegistry, MsgExecuteContract, MsgSnip20Send, MsgSnip20Transfer, SecretNetworkClient } = require("secretjs");


const OWNER_NFT_VK = "OWNER_NFT_VK"

// secret1j5e8yyzljq9c78tjlshtvefz8fslzupfy5l6r0: testnet

const NFT_ADDRESS = "secret199p24qyx23maqgsgt4ptujts309x5z4atfh5gg"

const USER_1_VK_ON_STAKING = "HUY_SOTA_VK_STAKING"

const main = async () => {
    // const grpcWebUrl = " https://secret-4.api.trivium.network:1317";
    // To create a readonly secret.js client, just pass in a gRPC-web endpoint
    // const secretjs = new CosmWasmClient(grpcWebUrl, undefined, BroadcastMode.Sync);

    const wallet = new Wallet(
        "grant rice replace explain federal release fix clever romance raise often wild taxi quarter soccer fiber love must tape steak together observe swap guitar",
    );

    // const owner = new Wallet(
    //     "joy clip vital cigar snap column control cattle ocean scout world rude labor gun find drift gaze nurse canal soldier amazing wealth valid runway"
    // );

    const owner = new Wallet(
        "live engage small salt donate memory admit sauce tenant ability beyond voyage"
    );

    const myAddress = wallet.address;
    const ownerAddress = owner.address;


    // // mainnet
    // const secretjs = await SecretNetworkClient.create({
    //     grpcWebUrl: "https://grpc.mainnet.secretsaturn.net",
    //     chainId: "secret-4",
    //     wallet: wallet,
    //     walletAddress: myAddress,
    // });

    // mainnet
    const secretjsOwner = await SecretNetworkClient.create({
        grpcWebUrl: "https://grpc.mainnet.secretsaturn.net",
        chainId: "secret-4",
        wallet: owner,
        walletAddress: ownerAddress,
    });

    const stakingCodeHash = await secretjsOwner.query.compute.contractCodeHash("secret1f4nnvjy7d3u07xdpjud50n4lms6xrqpzn28khe");

    // // transfer_nft
    // const transfer_nft_msg = new MsgExecuteContract({
    //     contractAddress: "secret1j5e8yyzljq9c78tjlshtvefz8fslzupfy5l6r0",
    //     msg: {
    //         transfer_nft: {
    //             recipient: myAddress,
    //             token_id: "LEGEN_DAO_1",
    //             memo: undefined,
    //             padding: undefined
    //         }
    //     },
    //     codeHash: nftCodeHash,
    //     sender: ownerAddress,
    //     sentFunds: []
    // })

     // transfer_nft
    const set_vk_msg = new MsgExecuteContract({
        contractAddress: "secret1f4nnvjy7d3u07xdpjud50n4lms6xrqpzn28khe",
        msg: {
            set_viewing_key: {
                key: USER_1_VK_ON_STAKING,
                padding: null,
            }
        },
        codeHash: stakingCodeHash,
        sender: ownerAddress,
        sentFunds: []
    })

    // const tx = await secretjs.tx.broadcast([set_status_msg], {
    //     gasLimit: 800000
    //     // gasPriceInFeeDenom: 0.000625
    // });

    // const tx = await secretjsOwner.tx.broadcast([set_vk_msg], {
    //     gasLimit: 800000,
    //     // gasPriceInFeeDenom: 0.000625
    // });

    // console.log(tx);

    // ================================================================== QUERY ==================================================================


    const total_locked = {}

    const balance = {
        address: ownerAddress,
        key: USER_1_VK_ON_STAKING,
    }


    const config = await secretjsOwner.query.compute.queryContract(
        {
            contractAddress: "secret1f4nnvjy7d3u07xdpjud50n4lms6xrqpzn28khe", 
            codeHash: stakingCodeHash, 
            query: {
                total_locked
            }
        }
    );

    console.log("config: ", config);
}

main()