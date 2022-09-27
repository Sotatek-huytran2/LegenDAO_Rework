const { Wallet, getMsgDecoderRegistry, MsgExecuteContract, MsgSnip20Send, MsgSnip20Transfer, SecretNetworkClient } = require("secretjs");


const OWNER_NFT_VK = "OWNER_NFT_VK"

const main = async () => {
    // const grpcWebUrl = " https://secret-4.api.trivium.network:1317";
    // To create a readonly secret.js client, just pass in a gRPC-web endpoint
    // const secretjs = new CosmWasmClient(grpcWebUrl, undefined, BroadcastMode.Sync);

    const wallet = new Wallet(
        "grant rice replace explain federal release fix clever romance raise often wild taxi quarter soccer fiber love must tape steak together observe swap guitar",
    );

    const owner = new Wallet(
        "joy clip vital cigar snap column control cattle ocean scout world rude labor gun find drift gaze nurse canal soldier amazing wealth valid runway"
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


    const secretjs = await SecretNetworkClient.create({
        grpcWebUrl: "https://grpc.testnet.secretsaturn.net",
        chainId: "pulsar-2",
        wallet: wallet,
        walletAddress: myAddress,
    });


    const secretjsOwner = await SecretNetworkClient.create({
        grpcWebUrl: "https://grpc.testnet.secretsaturn.net",
        chainId: "pulsar-2",
        wallet: owner,
        walletAddress: ownerAddress,
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

    // contract_addr: HumanAddr,
    //     /// If not specified, use all funds
    //     amount: Option<Uint128>,
    //     /// Probably not necessary
    //     memo: Option<String>,
    //     /// Wanted message to initiate at the destination contract (defined in the destination contract)
    //     msg: Binary,
    // const send_from_platform_msg = new MsgExecuteContract({
    //     contractAddress: "secret1xcdxl3pfv9n3kycfhxgtul27awjw3hd4k96mrg",
    //     msg: {
    //         send_from_platform: {
    //             amount: "10000000",
    //             contract_addr: "secret1lnk3x8q06nckmprptchh238gshzskkvjht0c8f",
    //             msg: Buffer.from(
    //                 JSON.stringify(
    //                     {
    //                         Deposit: {}
    //                     }
    //                 )
    //             ).toString("base64"),
    //             memo: "Try staking",
    //         }
    //     },
    //     codeHash: "2ae84f76f2411405ee17430446a0bb2754c1b70b91cea3820e37962a090374ab",
    //     sender: myAddress,
    //     sentFunds: []
    // })
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

    // const msg_set_viewing_key_for_staking = new MsgExecuteContract({
    //     contractAddress: "secret1lnk3x8q06nckmprptchh238gshzskkvjht0c8f",
    //     msg: {
    //         set_viewing_key: {
    //             key: "hello"
    //         }
    //     },
    //     codeHash: stakingCodeHash,
    //     sender: myAddress,
    //     sentFunds: []
    // })

    // const add_receiving_msg = new MsgExecuteContract({
    //     contractAddress: "secret1xcdxl3pfv9n3kycfhxgtul27awjw3hd4k96mrg",
    //     msg: {
    //         add_receiving_contracts: {
    //             addresses: [
    //                 "secret1lnk3x8q06nckmprptchh238gshzskkvjht0c8f"
    //             ]
    //         }
    //     },
    //     codeHash: platformCodeHash,
    //     sender: myAddress,
    //     sentFunds: []
    // });

    // const withdraw_msg = new MsgExecuteContract({
    //     contractAddress: "secret1lnk3x8q06nckmprptchh238gshzskkvjht0c8f",
    //     msg: {
    //         withdraw: {
    //             amount: "100"
    //         }
    //     },
    //     codeHash: stakingCodeHash,
    //     sender: myAddress,
    //     sentFunds: []
    // });


    // const msg = new MsgSnip20Transfer({
    //     contractAddress: "secret18y59n8z3frrslek52tkkq6t9yk76cdp57wztn9",
    //     msg: {
    //         transfer: {
    //             amount: "1",
    //             recipient: myAddress,
    //             padding: undefined
    //         }
    //     },
    //     codeHash,
    //     sender: myAddress,
    // });


    const nftCodeHash = await secretjsOwner.query.compute.contractCodeHash("secret1j5e8yyzljq9c78tjlshtvefz8fslzupfy5l6r0");

    // console.log(nftCodeHash)


    const set_status_msg = new MsgExecuteContract({
        contractAddress: "secret1j5e8yyzljq9c78tjlshtvefz8fslzupfy5l6r0",
        msg: {
            set_contract_status: {
                level: "normal",
                padding: undefined
            }
        },
        codeHash: nftCodeHash,
        sender: ownerAddress,
        setFunds: []
    })

    const mint_nft_msg  = new MsgExecuteContract({
        contractAddress: "secret1j5e8yyzljq9c78tjlshtvefz8fslzupfy5l6r0",
        msg: {
            mint_nft: {
                token_id: "LEGEN_DAO_2",
                owner: ownerAddress,
                public_metadata: {
                    token_uri: undefined,
                    extension: {
                        name: "NFT_2",
                        description: undefined,
                        image: "uri_2",
                        image_data: undefined,
                        external_url: "url_2",
                        attributes: [],
                        background_color: undefined,
                        animation_url: undefined,
                        youtube_url: undefined,
                        media: [],
                        protected_attributes: []
                    }
                },
                private_metadata: undefined,
                royalty_info: undefined,
                serial_number: undefined,
                memo: undefined,
                padding: undefined
            }
        },
        codeHash: nftCodeHash,
        sender: ownerAddress,
        sentFunds: []
    })

    const create_vk_msg = new MsgExecuteContract({
        contractAddress: "secret1j5e8yyzljq9c78tjlshtvefz8fslzupfy5l6r0",
        msg: {
            create_viewing_key: {
                entropy: OWNER_NFT_VK,
                padding: undefined
            }
        },
        codeHash: nftCodeHash,
        sender: ownerAddress,
        sentFunds: []
    })

    const set_vk_msg = new MsgExecuteContract({
        contractAddress: "secret1j5e8yyzljq9c78tjlshtvefz8fslzupfy5l6r0",
        msg: {
            set_viewing_key: {
                key: OWNER_NFT_VK,
                padding: undefined
            }
        },
        codeHash: nftCodeHash,
        sender: ownerAddress,
        sentFunds: []
    })


    // transfer_nft
    const transfer_nft_msg = new MsgExecuteContract({
        contractAddress: "secret1j5e8yyzljq9c78tjlshtvefz8fslzupfy5l6r0",
        msg: {
            transfer_nft: {
                recipient: myAddress,
                token_id: "LEGEN_DAO_1",
                memo: undefined,
                padding: undefined
            }
        },
        codeHash: nftCodeHash,
        sender: ownerAddress,
        sentFunds: []
    })

    // const tx = await secretjs.tx.broadcast([set_status_msg], {
    //     gasLimit: 800000
    //     // gasPriceInFeeDenom: 0.000625
    // });

    // const tx = await secretjsOwner.tx.broadcast([transfer_nft_msg], {
    //     gasLimit: 800000,
    //     // gasPriceInFeeDenom: 0.000625
    // });

    // console.log(tx);

    // ================================================================== QUERY ==================================================================

    const contract_config = {}

    const nft_info = {
        token_id: "LEGEN_DAO_1"
    }

    const owner_of = {
        token_id: "LEGEN_DAO_1",
        viewer: {
            address: ownerAddress,
            viewing_key: OWNER_NFT_VK
        },
        include_expired: undefined
    }

    const tokens = {
        owner: ownerAddress,
        viewer: undefined,
        viewing_key: OWNER_NFT_VK,
        start_after: "",
        limit: undefined,
    }



    const config = await secretjs.query.compute.queryContract(
        {
            contractAddress: "secret1j5e8yyzljq9c78tjlshtvefz8fslzupfy5l6r0", 
            codeHash: nftCodeHash, 
            query: {
                owner_of
            }
        }
    );

    console.log("config: ", config);
}

main()