const { Wallet, getMsgDecoderRegistry, MsgExecuteContract, MsgSnip20Send, MsgSnip20Transfer, SecretNetworkClient } = require("secretjs");

const { BigNumber } = require('bignumber.js');

const OWNER_NFT_VK = "OWNER_NFT_VK"

// secret1j5e8yyzljq9c78tjlshtvefz8fslzupfy5l6r0: testnet


const PLATFORM_ADDRESS = "secret1dcdvruu9r87weary8z9rdztyjzua7n8g4j5gdh"
const MINTING_ADDRESS = "secret1vxf87arx93zsm4hxkqp6nys3kr35r2r406qkzp"
const SNIP20_ADDRESS = "secret16xlsf4qz05ylyamstqudqppwpzy4hp4hre6sdg"
const NFT_ADDRESS = "secret1vs79386n0d0qqsv8nr6n8mypmk0tsc5lfgq7v5"

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

    const u8_private_key = owner.privateKey;

    console.log(Buffer.from(owner.publicKey).toString("base64"))

    console.log("=======")

    const private_key = Buffer.from(u8_private_key).toString("base64");
    console.log(private_key);

    // privatekey(hex): 7e80cf6267c0cdefe7e94929bbffde347bbad74958aee05b8c33a83d1211a9bd
    // privatekey(hex): 7e80cf6267c0cdefe7e94929bbffde347bbad74958aee05b8c33a83d1211a9bd

    // // mainnet
    // const secretjs = await SecretNetworkClient.create({
    //     grpcWebUrl: "https://grpc.mainnet.secretsaturn.net",
    //     chainId: "secret-4",
    //     wallet: wallet,
    //     walletAddress: myAddress,
    // });

    // // mainnet
    // const secretjsOwner = await SecretNetworkClient.create({
    //     grpcWebUrl: "https://grpc.mainnet.secretsaturn.net",
    //     chainId: "secret-4",
    //     wallet: owner,
    //     walletAddress: ownerAddress,
    // });

    

    //
    // const secretjs = await SecretNetworkClient.create({
    //     grpcWebUrl: "https://grpc.testnet.secretsaturn.net",
    //     chainId: "pulsar-2",
    //     wallet: wallet,
    //     walletAddress: myAddress,
    // });


    const secretjsOwner = await SecretNetworkClient.create({
        grpcWebUrl: "https://grpc.testnet.secretsaturn.net",
        chainId: "pulsar-2",
        wallet: owner,
        walletAddress: ownerAddress,
        //encryptionSeed: 
    });


    // ==============================================================

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

    
    const platformCodeHash = await secretjsOwner.query.compute.contractCodeHash(PLATFORM_ADDRESS);
    const mintingCodeHash = await secretjsOwner.query.compute.contractCodeHash(MINTING_ADDRESS);
    const snip20CodeHash = await secretjsOwner.query.compute.contractCodeHash(SNIP20_ADDRESS);
    const sip721CodeHash = await secretjsOwner.query.compute.contractCodeHash(NFT_ADDRESS);
    
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

    const add_receiving_msg = new MsgExecuteContract({
        contractAddress: PLATFORM_ADDRESS,
        msg: {
            add_receiving_contracts: {
                addresses: [
                    MINTING_ADDRESS
                ]
            }
        },
        codeHash: platformCodeHash,
        sender: ownerAddress,
        sentFunds: []
    });

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


    const nftCodeHash = await secretjsOwner.query.compute.contractCodeHash(NFT_ADDRESS);

    // console.log(nftCodeHash)

    const set_status_msg = new MsgExecuteContract({
        contractAddress: NFT_ADDRESS,
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
        contractAddress: NFT_ADDRESS,
        msg: {
            mint_nft: {
                token_id: "LEGEN_DAO_5",
                owner: ownerAddress,
                public_metadata: {
                    token_uri: undefined,
                    extension: {
                        name: "NFT_5",
                        description: undefined,
                        image: "uri_5",
                        image_data: undefined,
                        external_url: "url_5",
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
                token_type: "avatar",
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
        contractAddress: NFT_ADDRESS,
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
        contractAddress: NFT_ADDRESS,
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
        contractAddress: NFT_ADDRESS,
        msg: {
            transfer_nft: {
                recipient: myAddress,
                token_id: "LEGEN_DAO_2",
                memo: undefined,
                padding: undefined
            }
        },
        codeHash: nftCodeHash,
        sender: ownerAddress,
        sentFunds: []
    })

    const batch_transfer_nft_msg = new MsgExecuteContract({
        contractAddress: NFT_ADDRESS,
        msg: {
            batch_transfer_nft: {
                transfers: [
                    {
                        recipient: myAddress,
                        token_ids: ["LEGEN_DAO_1"],
                        memo: undefined,
                    },
                    {
                        recipient: myAddress,
                        token_ids: ["LEGEN_DAO_3"],
                        memo: undefined,
                    },
                ],
                padding: undefined
            }
        },
        codeHash: nftCodeHash,
        sender: ownerAddress,
        sentFunds: []
    })

    const add_minter_msg = new MsgExecuteContract({
        contractAddress: NFT_ADDRESS,
        msg: {
            add_minters: {
                minters: [PLATFORM_ADDRESS, MINTING_ADDRESS],
                padding: undefined
            }
        },
        codeHash: nftCodeHash,
        sender: ownerAddress,
        sentFunds: []
    })

    //  ================================================== MINTING CONTRACT INTERACT 

    const change_minting_state_msg = new MsgExecuteContract({
        contractAddress: MINTING_ADDRESS,
        msg: {
            changing_minting_state: {
                mint_state: 3,
                cap_amount: undefined
            }
        },
        codeHash: mintingCodeHash,
        sender: ownerAddress,
        sentFunds: []
    })


    // ========================================== SNIP 20 interact

    const AMOUNT_MINT = new BigNumber(3).multipliedBy(new BigNumber(10).pow(6));

    const msg_deposit = {
        deposit: {
            to: ownerAddress
        }
    }


    const send_token_to_platform_msg = new MsgExecuteContract({
        contractAddress: SNIP20_ADDRESS,
        msg: {
            send: {
                "recipient": PLATFORM_ADDRESS,
                "recipient_code_hash": platformCodeHash,
                "amount": AMOUNT_MINT.toFixed(),
                "msg": Buffer.from(JSON.stringify(msg_deposit)).toString("base64"),
                "memo": "",
                "padding": null,
            }
        },
        codeHash: snip20CodeHash,
        sender: ownerAddress,
        sentFunds: []
    })


    //  ================================================== PLATFORM CONTRACT INTERACT 

    

    const platform_deposit_minting = {
        mint: {
            mint_for: ownerAddress,
            amount_avatar_to_mint: 1,
            amount_loot_box_to_mint: 1,
            amount_item_to_mint: 1,
        },
    }


    const mint_from_paltform_msg = new MsgExecuteContract({
        contractAddress: PLATFORM_ADDRESS,
        msg: {
            send_from_platform: {
                contract_addr: MINTING_ADDRESS,
                amount: AMOUNT_MINT.toFixed(),
                msg: Buffer.from(JSON.stringify(platform_deposit_minting)).toString("base64"),
                memo: "",
            }
        },
        codeHash: platformCodeHash,
        sender: ownerAddress,
        sentFunds: []
    })


    // const tx = await secretjs.tx.broadcast([set_status_msg], {
    //     gasLimit: 800000
    //     // gasPriceInFeeDenom: 0.000625
    // });

    // const tx = await secretjsOwner.tx.broadcast([mint_from_paltform_msg], {
    //     gasLimit: 8000000,
    //     // gasPriceInFeeDenom: 0.000625
    // });

    // console.log(tx);

    // ================================================================== QUERY ==================================================================

    const contract_config = {}

    const nft_info = {
        token_id: "LEGEN_DAO_1"
    }

    const owner_of = {
        token_id: "LEGEN_DAO_3",
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
        start_after: undefined,
        limit: undefined,
    }

    // const config = await secretjsOwner.query.compute.queryContract(
    //     {
    //         contractAddress: NFT_ADDRESS,
    //         codeHash: nftCodeHash, 
    //         query: {
    //             owner_of
    //         }
    //     }
    // );

    // console.log("config: ", config);
}

main()