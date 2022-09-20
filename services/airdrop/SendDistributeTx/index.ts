import { AzureFunction, Context, HttpRequest } from "@azure/functions";
import mongoose, { HydratedDocument } from "mongoose";
import { SecretNetworkClient, Tx, TxResultCode, Wallet } from "secretjs";
import { ClaimDocument, ClaimModel } from "../shared/models/ClaimModel";

import {
    MONGODB_NAME,
    MONGODB_URL,
    ADMIN_MNEMONIC,
    ADMIN_ADDRESS,
    GRPC_NODE_ADDRESS,
    CHAIN_ID,
    AIRDROP_CONTRACT_ADDRESS,
    AIRDROP_CONTRACT_HASH,
} from "../shared/consts";

class ComputeError extends Error {
    tx: Tx;

    constructor(tx: Tx, message: string) {
        super(message);
        this.name = "ComputeError";
        this.tx = tx;
    }
}

interface AirdropEntry {
    address: string;
    to: string;
    amount: string;
}

interface ConfirmAirdropMsg {
    confirm_airdrop: { airdrops: AirdropEntry[] };
}

// *************** HELPER FUNCTIONS  ********** //
const cleanup = async () => {
    await mongoose.disconnect();
};

const initSecretClient = async (): Promise<SecretNetworkClient> => {
    let walletProto = new Wallet(ADMIN_MNEMONIC);

    console.log(`creating client to ${CHAIN_ID} @ ${GRPC_NODE_ADDRESS} for ${ADMIN_ADDRESS}`);
    return await SecretNetworkClient.create({
        grpcWebUrl: GRPC_NODE_ADDRESS,
        wallet: walletProto,
        walletAddress: ADMIN_ADDRESS,
        chainId: CHAIN_ID,
    });
};

// Secret Contract gas costs to release the airdrops.
// The figures are based on experimentation + a small safety buffer.
let AIRDROP_BASE_COST = 63_000; // gas
let AIRDROP_ITEM_COST = 3_800; //gas

const createSendTx = async (client: SecretNetworkClient, airdrops: AirdropEntry[]) => {
    let msg: ConfirmAirdropMsg = {
        confirm_airdrop: { airdrops },
    };

    console.log(`sending the confirm_airdrops message: ${JSON.stringify(msg)}`);
    let tx = await client.tx.compute.executeContract(
        {
            contractAddress: AIRDROP_CONTRACT_ADDRESS,
            codeHash: AIRDROP_CONTRACT_HASH,
            sender: client.address,
            msg,
        },
        {
            // based on experimentation
            gasLimit: 63000 + airdrops.length * 3800,
            gasPriceInFeeDenom: 0.25,
        },
    );

    if (tx.code !== TxResultCode.Success) {
        console.log(JSON.stringify(tx.jsonLog || tx));
        throw new ComputeError(tx, `Failed to run confirm_airdrops`);
    }

    let addresses = tx.arrayLog.filter((log) => log.key == "airdropped_to").map((log) => log.value);
    let logs = JSON.stringify(addresses || tx.rawLog);
    console.log(`confirm_airdrops sent successfully to: ${tx.transactionHash} ${logs}`);
    return tx;
};

// *************** MAIN ********** //

const timerTrigger: AzureFunction = async function (context: Context, myTimer: any): Promise<void> {
    await mongoose.connect(MONGODB_URL, {
        dbName: MONGODB_NAME,
        maxPoolSize: 10,
    });

    // get some pending txs

    let maxTxGas = 1_000_000;

    // ~245 items per 1M gas
    let maxAirdropCount = Math.floor((maxTxGas - AIRDROP_BASE_COST) / AIRDROP_ITEM_COST);

    let claims = await ClaimModel.find({ status: "Submitted" }).limit(maxAirdropCount);

    if (claims.length === 0) {
        console.log("no airdrops to approve");
        return;
    }

    let drops: AirdropEntry[] = [];

    for (const claim of claims) {
        drops.push({
            address: claim.address,
            to: claim.recipient,
            amount: claim.amount.toString(),
        });
    }

    let client = await initSecretClient();

    await createSendTx(client, drops);

    for (const claim of claims) {
        claim.status = "Claimed";
    }

    console.log("Setting all confirmed airdrops to status = Claimed");
    await Promise.all(claims.map((claim) => claim.save()));

    await cleanup();
};

export default timerTrigger;
