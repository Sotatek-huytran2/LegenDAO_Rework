const { Contract, getAccountByName, getLogs } = require("secret-polar-reworks");



const PRNG_SEED_KEY = "LEGENDAO_ENTROPY"
const INDEX = "23"
const INIT_SNIP20_LABEL = "Instantiate config ".concat(INDEX)
const INIT_NFT_LABEL = "Instantiate config snip-721 ".concat(INDEX)
const INIT_MINTING_LABEL = "Instantiate config nft-minting ".concat(INDEX)

async function run() {
  const contract_owner = getAccountByName("huy_sota");
  //const contract_snip721 = new Contract("snip721");
  const contract_minting = new Contract("minter-contract");
  //const contract_snip20 = new Contract("snip20");


  //await contract_snip721.parseSchema();
  await contract_minting.parseSchema();
  //await contract_snip20.parseSchema();

  //console.log(contract_owner.account.address);


  //// mainnet
  // const deploy_response = await contract.deploy(
  //   contract_owner,
  //   { // custom fees
  //     amount: [{ amount: "50000", denom: "uscrt" }],
  //     gas: "5000000",
  //   }
  // );

  // const deploy_snip20_response = await contract_snip20.deploy(
  //   contract_owner,
  //   { // custom fees
  //     amount: [{ amount: "50000", denom: "uscrt" }],
  //     gas: "3000000",
  //   }
  // );

  // console.log(deploy_snip20_response);

  // const lgndInitMsg = {
  //   prng_seed: "YWE",
  //   symbol: "LGND",
  //   name: "legend",
  //   decimals: 6,
  //   initial_balances: [
  //     { address: contract_owner.account.address, amount: "10000000000000000" },
  //   ],
  //   config: {
  //     public_total_supply: true,
  //     enable_deposit: true,
  //     enable_redeem: true,
  //     enable_mint: true,
  //     enable_burn: true,
  //   },
  //   supported_denoms: [process.env.LGND_NATIVE],
  // };


  // const resp_snip20 = await contract_snip20.instantiate(
  //   lgndInitMsg,
  //   INIT_SNIP20_LABEL,
  //   contract_owner
  // );

  // console.log(resp_snip20);

  // // 721
  // // =========================================================
  // console.log("=========================================================");

  // const deploy_response_snip721 = await contract_snip721.deploy(
  //   contract_owner,
  //   { // custom fees
  //     amount: [{ amount: "750000", denom: "uscrt" }],
  //     gas: "50000000",
  //   }
  // );

  // console.log(deploy_response_snip721);

  // const nftInitMsg = {
  //   name: "LegenDAO NFT",
  //   entropy: PRNG_SEED_KEY,
  //   symbol: "NFT"
  // };


  // const resp_snip721 = await contract_snip721.instantiate(
  //   nftInitMsg,
  //   INIT_NFT_LABEL,
  //   contract_owner
  // );

  // console.log(resp_snip721);

  // // mintin
  // // =========================================================
  // console.log("=========================================================");


  const deploy_minting_response = await contract_minting.deploy(
    contract_owner,
    { // custom fees
      amount: [{ amount: "750000", denom: "uscrt" }],
      gas: "50000000",
    }
  );

  console.log(deploy_minting_response);


  const nftMintingInitMsg = {
    nft_count: 400,
    nft_contract:
    {
      // address: contract_snip721.contractAddress,
      // hash: deploy_response_snip721.contractCodeHash
      address: "secret1vs79386n0d0qqsv8nr6n8mypmk0tsc5lfgq7v5",
      hash: "10f08f81497f3eb9ebdeda8e5986dfddb8eeffd8af24a8a6f86d6ea5fde9f584"
    },
    base_uri: "https://concak.com/",
    random_seed: Buffer.from(JSON.stringify("ABC")).toString("base64"),
    price: [
      {
        price: "1000000",
        whitelist_price: "1000000",
        items_price: "1000000",
        loot_box_price: "1000000",
        token:
        {
          snip20:
          {
            // address: contract_snip20.contractAddress,
            // hash: deploy_snip20_response.contractCodeHash
            address: "secret16xlsf4qz05ylyamstqudqppwpzy4hp4hre6sdg",
            hash: "61d8a71482b8d6fdc8be79d0911fd5ff0304d5ece9d8c56e68690e258239f9e7"
          }
        },
      }
    ],
    platform: {
      address: "secret1dcdvruu9r87weary8z9rdztyjzua7n8g4j5gdh",
      hash: "a74e45a26a27330410785c86a07f9c8cebcf59b0d912e8bbaa2e6904f0b2bd40"
    },
  }

  const resp_minting = await contract_minting.instantiate(
    nftMintingInitMsg,
    INIT_MINTING_LABEL,
    contract_owner,
    null,
    { // custom fees
      amount: [{ amount: "750000", denom: "uscrt" }],
      gas: "6000000",
    }
  );

  console.log(resp_minting);



}

module.exports = { default: run };









// ======================================================================



