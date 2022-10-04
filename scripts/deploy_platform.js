const { Contract, getAccountByName, getLogs } = require("secret-polar-reworks");

const { CosmWasmClient, BroadcastMode } = require("secretjs");

const VIEWING_KEY = "hello";


async function run() {
    const contract_owner = getAccountByName("huy_sota");

    // const contract_platform = new Contract("platform");

    // // console.log(contract_platform)
    // // await contract_platform.parseSchema();

    // // //console.log(contract_owner.account.address);

    // const deploy_response = await contract_platform.deploy(
    //     contract_owner,
    //     { // custom fees
    //         amount: [{ amount: "50000", denom: "uscrt" }],
    //         gas: "3000000",
    //     }
    // );

    // console.log(deploy_response);

    const lgndToken = 'secret18y59n8z3frrslek52tkkq6t9yk76cdp57wztn9';
    
    // const grpcWebUrl = "http://testnet.securesecrets.org:1317/";
    const grpcWebUrl = "https://secret-4.api.trivium.network:1317/";

    // To create a readonly secret.js client, just pass in a gRPC-web endpoint
    const secretjs = new CosmWasmClient(grpcWebUrl, undefined, BroadcastMode.Sync);

    const lgndContractHash = await secretjs.getCodeHashByContractAddr(lgndToken);

    // const platformInitMsg = {
    //     token: {
    //         address: lgndToken,
    //         hash: lgndContractHash,
    //     },
    //     token_native_denom: process.env.LGND_NATIVE,
    //     viewing_key: VIEWING_KEY,
    // };


    // const resp = await contract_platform.instantiate(
    //     platformInitMsg,
    //     "Instantiate config platform 6",
    //     contract_owner
    // );

    // console.log(resp);


    // console.log("================================================================");


    const contract_staking = new Contract("staking");
    //await contract_staking.parseSchema();

    //console.log(contract_owner.account.address);

    const deploy_response_stake = await contract_staking.deploy(
        contract_owner,
        { // custom fees
            amount: [{ amount: "50000", denom: "uscrt" }],
            gas: "3000000",
        }
    );

    console.log(deploy_response_stake);


    const stakingInitMsg = {
        token: {
            address: lgndToken,
            hash: lgndContractHash,
        },
        platform: {
            // address: resp.contractAddress,
            //hash: deploy_response.contractCodeHash,
            address: "secret1xcdxl3pfv9n3kycfhxgtul27awjw3hd4k96mrg",
            hash: "2ae84f76f2411405ee17430446a0bb2754c1b70b91cea3820e37962a090374ab",
        },
        inflation_schedule: [{ end_block: 10_000_000, reward_per_block: "10000" }],
        viewing_key: VIEWING_KEY,
        prng_seed: "IAo=",
    }


    const resp_stake = await contract_staking.instantiate(
        stakingInitMsg,
        "Instantiate config staking 11",
        contract_owner
    );

    console.log(resp_stake);

    console.log("================================================================");

    console.log("Success!");
    console.log(`$LGND address: ${lgndToken}`);
    // console.log(`Platform address: ${resp.contractAddress}`);
    console.log(`Staking address: ${resp_stake.contractAddress}`);



    // const contract_info = await contract.instantiate({"count": 102}, "deploy test", contract_owner);
    // console.log(contract_info);

    // // use below line if contract initiation done using another contract
    // // const contract_addr = "secret76597235472354792347952394";
    // // contract.instantiatedWithAddress(contract_addr);

    // const inc_response = await contract.tx.increment({account: contract_owner});
    // console.log(inc_response);
    // // to get logs as a key:value object
    // // console.log(getLogs(inc_response));

    // const response = await contract.query.get_count();
    // console.log(response);

    // const transferAmount = [{"denom": "uscrt", "amount": "15000000"}] // 15 SCRT
    // const customFees = { // custom fees
    //   amount: [{ amount: "750000", denom: "uscrt" }],
    //   gas: "3000000",
    // }
    // const ex_response = await contract.tx.increment(
    //   {account: contract_owner, transferAmount: transferAmount}
    // );
    // // const ex_response = await contract.tx.increment(
    // //   {account: contract_owner, transferAmount: transferAmount, customFees: customFees}
    // // );
    // console.log(ex_response);
}

module.exports = { default: run };


// "secretjs": "1.4.0-alpha.10"

// $LGND address: secret16gg80l22ft4nyzv5jnp7e2yjkdqvzune3aa948
// Platform address: secret10x7ng3j4m636wmx5ssxdafpx0exk9p6c3qznv2
// Staking address: secret165m55asf7j28mwn7e627m46kzl088gglythez7


//   {
//     codeId: 707,
//     contractCodeHash: '2ae84f76f2411405ee17430446a0bb2754c1b70b91cea3820e37962a090374ab',
//     deployTimestamp: 'Tue Sep 20 2022 16:24:15 GMT+0700 (Indochina Time)'
//   }
//   Instantiating with label: Instantiate config platform 6
//   {
//     contractAddress: 'secret1xcdxl3pfv9n3kycfhxgtul27awjw3hd4k96mrg',
//     instantiateTimestamp: 'Tue Sep 20 2022 16:24:16 GMT+0700 (Indochina Time)'
//   }
//   ================================================================
//   Creating compressed .wasm file for staking
//   {
//     codeId: 708,
//     contractCodeHash: 'af7b96f199defbd6991cc3673d65ebb6b97f065a93e5cd0c456c64658e504b55',
//     deployTimestamp: 'Tue Sep 20 2022 16:24:49 GMT+0700 (Indochina Time)'
//   }
//   Instantiating with label: Instantiate config staking 6
//   {
//     contractAddress: 'secret15wg0wjxf6susw2gdgaw4csnpzxrnpgf8h8mw9n',
//     instantiateTimestamp: 'Tue Sep 20 2022 16:24:49 GMT+0700 (Indochina Time)'
//   }
  

//{
//     codeId: 721,
//     contractCodeHash: 'cd4f88cc0c49e064b1862c5414eb812c28abc51a82a990929db57465f81835c3',
//     deployTimestamp: 'Fri Sep 30 2022 10:27:15 GMT+0700 (Indochina Time)'
//   }
//   Instantiating with label: Instantiate config staking 9
//   {
//     contractAddress: 'secret136fjmfhf6qqy0yyg0hhzexmesr5mt7tna9w8q0',
//     instantiateTimestamp: 'Fri Sep 30 2022 10:28:39 GMT+0700 (Indochina Time)'
//   }
  

// {
//     codeId: 726,
//     contractCodeHash: 'cd4f88cc0c49e064b1862c5414eb812c28abc51a82a990929db57465f81835c3',
//     deployTimestamp: 'Mon Oct 03 2022 16:58:50 GMT+0700 (Indochina Time)'
//   }
//   Instantiating with label: Instantiate config staking 11
//   {
//     contractAddress: 'secret1f4nnvjy7d3u07xdpjud50n4lms6xrqpzn28khe',
//     instantiateTimestamp: 'Mon Oct 03 2022 17:00:46 GMT+0700 (Indochina Time)'
//   }
  


// mainnet
// $LGND address: secret18y59n8z3frrslek52tkkq6t9yk76cdp57wztn9
// Platform address: secret1xcdxl3pfv9n3kycfhxgtul27awjw3hd4k96mrg
// Staking address: secret15wg0wjxf6susw2gdgaw4csnpzxrnpgf8h8mw9n

