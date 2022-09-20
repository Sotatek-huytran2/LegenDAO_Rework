const axios = require("axios");
const { Wallet, SecretNetworkClient, TxResultCode } = require("secretjs");

const { getScrtBalance, ComputeError } = require("./utils");

const fs = require("fs");

const { Bip39, Random } = require("@iov/crypto");

const customFees = {
    exec: {
        amount: [{ amount: "400000", denom: "uscrt" }],
        gas: "1600000",
    },
    init: {
        amount: [{ amount: "2500000", denom: "uscrt" }],
        gas: "10000000",
    },
    upload: {
        amount: [{ amount: "2500000", denom: "uscrt" }],
        gas: "10000000",
    },
};

const getFromFaucet = async (address) => {
    console.log(`address=${address}`);
    await axios.get(`http://localhost:5000/faucet?address=${address}`);
};

async function fillUpFromFaucet(client, targetBalance) {
    let balance = await getScrtBalance(client);
    while (Number(balance) < targetBalance) {
        try {
            await getFromFaucet(client.address);
        } catch (e) {
            console.error(`failed to get tokens from faucet: ${e}`);
        }
        balance = await getScrtBalance(client);
    }
    console.error(`got tokens from faucet: ${balance}`);
    return balance;
}

const createAccount = async () => {
    // Create random address and mnemonic
    const mnemonic = Bip39.encode(Random.getBytes(16)).toString();

    const wallet = new Wallet(mnemonic);
    const accAddress = wallet.address;

    // console.log(`acc: ${accAddress}`)

    // // This wraps a single keypair and allows for signing.
    // const signingPen = await Secp256k1Pen.fromMnemonic(mnemonic);
    //
    // // Get the public key
    // const pubkey = encodeSecp256k1Pubkey(signingPen.pubkey);
    //
    // // Get the wallet address
    // const accAddress = pubkeyToAddress(pubkey, 'secret');
    //
    // // Query the account
    // const client = new CosmWasmClient(restEndpoint);
    // const account = await client.getAccount(accAddress);

    console.log("mnemonic: ", mnemonic);
    console.log("address: ", accAddress);

    return [mnemonic, accAddress];
};

const Instantiate = async (client, initMsg, codeId, codeHash, label) => {
    const [contractAddress] = await InstantiateWithHeight(client, initMsg, codeId, codeHash, label);

    return contractAddress;
};

const InstantiateWithHeight = async (client, initMsg, codeId, codeHash, label) => {
    //todo remove
    console.log("calling instantiatewithheight")
    const tx = await client.tx.compute.instantiateContract(
        {
            sender: client.address,
            codeId,
            codeHash,
            initMsg,
            label: label || "My Counter" + Math.ceil(Math.random() * 1000000),
        },
        {
            gasLimit: 5_000_000,
            gasPriceInFeeDenom: 0.25,
            broadcastCheckIntervalMs: process.env.TX_CHECK_MS
        },
    );
    console.log(`gas used for instantiate: ${tx.gasUsed}`);
    try {
        if (tx.arrayLog === undefined) {
            throw Error("contract instantiation failed, result:")
        }
        const contractAddress = tx.arrayLog.find(
            (log) => log.type === "message" && log.key === "contract_address",
        ).value;

        console.log(`Address: ${contractAddress}`);

        return [contractAddress, tx.height];
    } catch (e) {
        console.log(`Error: ${e}`);
        console.log(JSON.stringify(tx));
    }
};

const storeCode = async (path, client) => {
    const wasm = fs.readFileSync(path);
    console.log("Uploading contract");
    const uploadReceipt = await client.tx.compute.storeCode(
        {
            wasmByteCode: wasm,
            sender: client.address,
            source: "",
            builder: "",
        },
        {
            gasLimit: 5_000_000,
            gasPriceInFeeDenom: 0.25,
            broadcastCheckIntervalMs: process.env.TX_CHECK_MS
        },
    );

    if (uploadReceipt.code !== TxResultCode.Success) {
        console.log(`Failed to get code id: ${uploadReceipt.rawLog}`);
        throw new ComputeError(uploadReceipt, `Failed to upload contract`);
    }

    const codeIdKv = uploadReceipt.jsonLog[0].events[0].attributes.find((a) => {
        return a.key === "code_id";
    });

    const codeId = Number(codeIdKv.value);
    console.log("codeId: ", codeId);

    const contractCodeHash = await client.query.compute.codeHash(codeId);
    console.log(`Contract hash: ${contractCodeHash}`);

    return [codeId, contractCodeHash];
};

const createCli = async (mnemonic, rest_endpoint, chain_id) => {
    let url = new URL(rest_endpoint);
    url.port = "9091";
    rest_endpoint = url.toString();

    const wallet = new Wallet(mnemonic);
    const accAddress = wallet.address;

    return await SecretNetworkClient.create({
        grpcWebUrl: rest_endpoint,
        chainId: chain_id,
        wallet: wallet,
        walletAddress: accAddress,
    });

    // return new SigningCosmWasmClient(
    //     rest_endpoint,
    //     accAddress,
    //     (data) => signingPen.sign(data),
    //     signingPen.privkey,
    //     customFees
    // );
};

module.exports = {
    getFromFaucet,
    fillUpFromFaucet,
    createAccount,
    storeCode,
    createCli,
    Instantiate,
    InstantiateWithHeight,
};
