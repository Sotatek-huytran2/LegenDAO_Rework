// require("dotenv").config();

import { Permission } from "secretjs";

export const MONGODB_NAME: string = process.env.MONGODB_NAME || "legendaoAirdropTest";
export const MONGODB_URL: string =
    process.env.MONGODB_URL ||
    "mongodb+srv://lgndTest:D24Y3XyIGo8B1Xz8@bridge-testnet.ekhng.mongodb.net/";

export const ADMIN_MNEMONIC: string = process.env.ADMIN_MNEMONIC || "";
export const ADMIN_ADDRESS: string = process.env.ADMIN_ADDRESS || "";

export const GRPC_NODE_ADDRESS: string = process.env.GRPC_NODE_ADDRESS || "http://localhost:9091";
export const CHAIN_ID: string = process.env.CHAIN_ID || "secretdev-1";

export const AIRDROP_CONTRACT_ADDRESS: string = process.env.AIRDROP_CONTRACT_ADDRESS || "";
export const AIRDROP_CONTRACT_HASH: string = process.env.AIRDROP_CONTRACT_HASH || "";

// address of the ...?
export const PERMIT_CONTRACT_ADDRESS: string = process.env.PERMIT_CONTRACT_ADDRESS || "secret1asdf";
export const ALLOWED_PERMISSION: Permission = "owner";
