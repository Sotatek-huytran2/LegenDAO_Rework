const { getAccountByName } = require("secret-polar-reworks");

const { CosmWasmClient, SigningCosmWasmClient, BroadcastMode, Secp256k1Pen } = require("secretjs");

const VIEWING_KEY = "hello";

async function run() {
    const contract_owner = getAccountByName("huy_sota");

    // contract_owner.account.mnemonic

    const wallet = await Secp256k1Pen.fromMnemonic(contract_owner.account.mnemonic);

    const lgndToken = 'secret1w9p38mejmkn3rn6l8erkxumrw46jcjfkgzzp00';
    
    // const grpcWebUrl = "http://testnet.securesecrets.org:1317/";
    const grpcWebUrl = "https://api.scrt.network/";
    // To create a readonly secret.js client, just pass in a gRPC-web endpoint
    const secretjs = new CosmWasmClient(grpcWebUrl, undefined, BroadcastMode.Sync);

    const client = new SigningCosmWasmClient(
        grpcWebUrl,
        contract_owner.account.address,
        signBytes => wallet.sign(signBytes)
    )
    
    console.log(client);

    console.log("================================================================");

    // // const tx = await client.sendTokens("secret1knm0adw6fk536hy24t8dk059yl0jz924uu9wqz", [{ amount: "7890", denom: "uscrt" }], "for dinner");

    const msg = {
        create_viewing_key: {
            entropy: "huytran",
            padding: null,
        }
    }

    const tx = await client.execute(lgndToken, msg);

    const data = new TextDecoder().decode(tx.data);
    
    console.log(tx);
    console.log(data);
}

module.exports = { default: run };


// $LGND address: secret16gg80l22ft4nyzv5jnp7e2yjkdqvzune3aa948
// Platform address: secret10x7ng3j4m636wmx5ssxdafpx0exk9p6c3qznv2
// Staking address: secret165m55asf7j28mwn7e627m46kzl088gglythez7


// secret16gg80l22ft4nyzv5jnp7e2yjkdqvzune3aa948|api_key_a+1u+GeH3Kg8EIfmG/kI1FZixEcfZXi93iKDX/V4ooM=

// secret16gg80l22ft4nyzv5jnp7e2yjkdqvzune3aa948|api_key_AWaELunwyCMJJNO+ZGDXED0yz6fSkLAC8RRBbZW/1hA=


/// mainnet 
