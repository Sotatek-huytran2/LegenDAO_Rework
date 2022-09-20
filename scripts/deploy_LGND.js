const { Contract, getAccountByName, getLogs } = require("secret-polar-reworks");

async function run () {
  const contract_owner = getAccountByName("huy_sota");
  const contract = new Contract("snip20");
  await contract.parseSchema();

  //console.log(contract_owner.account.address);

  const deploy_response = await contract.deploy(
    contract_owner,
    { // custom fees
      amount: [{ amount: "50000", denom: "uscrt" }],
      gas: "3000000",
    }
  );

  console.log(deploy_response);

  const lgndInitMsg = {
    prng_seed: "YWE",
    symbol: "LGND",
    name: "legend",
    decimals: 6,
    initial_balances: [
        {address: contract_owner.account.address, amount: "10000000000000000"},
    ],
    config: {
        public_total_supply: true,
        enable_deposit: true,
        enable_redeem: true,
        enable_mint: true,
        enable_burn: true,
    },
    supported_denoms: [process.env.LGND_NATIVE],
};


  const resp = await contract.instantiate(
    lgndInitMsg,
    "Instantiate config 2",
    contract_owner
  );

  console.log(resp);

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


// {
//     codeId: 12688,
//         contractCodeHash: '61d8a71482b8d6fdc8be79d0911fd5ff0304d5ece9d8c56e68690e258239f9e7',
//             deployTimestamp: 'Mon Sep 19 2022 16:31:48 GMT+0700 (Indochina Time)'
// }
//   Instantiating with label: Instantiate config 1
// {
//     contractAddress: 'secret16gg80l22ft4nyzv5jnp7e2yjkdqvzune3aa948',
//         instantiateTimestamp: 'Mon Sep 19 2022 16:32:07 GMT+0700 (Indochina Time)'
// }

// Code-ID: 706
// Mainnet:
//           61d8a71482b8d6fdc8be79d0911fd5ff0304d5ece9d8c56e68690e258239f9e7
// CodeHash: 61d8a71482b8d6fdc8be79d0911fd5ff0304d5ece9d8c56e68690e258239f9e7
// Address: secret18y59n8z3frrslek52tkkq6t9yk76cdp57wztn9 (LegenDAO-1)
// Address: secret1w9p38mejmkn3rn6l8erkxumrw46jcjfkgzzp00 (LegenDAO-2)
  