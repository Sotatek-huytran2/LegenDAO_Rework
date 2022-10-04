const { Contract, getAccountByName, getLogs } = require("secret-polar-reworks");
 


const PRNG_SEED_KEY = "LEGENDAO_ENTROPY"

async function run() {
  const contract_owner = getAccountByName("huy_sota");
  const contract = new Contract("snip721");
  await contract.parseSchema();
 
  //console.log(contract_owner.account.address);
 
  const deploy_response = await contract.deploy(
    contract_owner,
    { // custom fees
      amount: [{ amount: "50000", denom: "uscrt" }],
      gas: "5000000",
    }
  );
 
  console.log(deploy_response);
 
  const nftInitMsg = {
    name: "LegenDAO NFT",
    entropy: PRNG_SEED_KEY,
    symbol: "NFT"
  };
 
 
  const resp = await contract.instantiate(
    nftInitMsg,
    "Instantiate NFT 5",
    contract_owner
  );
 
  console.log(resp);
}
 
module.exports = { default: run };
 
 
 
// test net 
// {
//   codeId: 13325,
//   contractCodeHash: '43b975db26414cd0649ff28c84dbfaf06e400e940b10324ff29e6f7daa731223',
//   deployTimestamp: 'Sun Sep 25 2022 16:30:08 GMT+0700 (Indochina Time)'
// }
// Instantiating with label: Instantiate NFT 2
// {
//   contractAddress: 'secret1j5e8yyzljq9c78tjlshtvefz8fslzupfy5l6r0',
//   instantiateTimestamp: 'Sun Sep 25 2022 16:30:08 GMT+0700 (Indochina Time)'
// }


// mainnet
// {
//   codeId: 718,
//   contractCodeHash: '4caf10eb45b7528af9da9fed0c1f9575dba9431305195ef0f3c23e808fe74ac6',
//   deployTimestamp: 'Wed Sep 28 2022 15:21:45 GMT+0700 (Indochina Time)'
// }
// Instantiating with label: Instantiate NFT 5
// {
//   contractAddress: 'secret199p24qyx23maqgsgt4ptujts309x5z4atfh5gg',
//   instantiateTimestamp: 'Wed Sep 28 2022 15:21:45 GMT+0700 (Indochina Time)'
// }

