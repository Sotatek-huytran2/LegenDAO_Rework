import mongoose from "mongoose";
import { Permit } from "../permit";

const MONGODB_COLLECTION = process.env.MONGODB_COLLECTION || "claims";

export type CLAIM_STATUS = "NotClaimed" | "Claimed" | "Submitted" | "NotWhitelisted";

export const permitSchema = new mongoose.Schema<Permit>({
    params: {
        permit_name: String,
        allowed_tokens: Array(String),
        chain_id: String,
        permissions: Array(String),
    },
    signature: {
        pub_key: {
            // A workaround making sure that the `type` field of `pub_key` is treated correctly
            // https://mongoosejs.com/docs/schematypes.html#type-key
            type: { type: String },
            value: String,
        },
        signature: String,
    },
});

export interface ClaimDocument {
    address: string;
    recipient: string;
    permit?: Permit;
    amount: mongoose.Types.Decimal128;
    status: CLAIM_STATUS;
}

export const claimSchema = new mongoose.Schema<ClaimDocument>(
    {
        address: { type: String, required: true, index: true, unique: true },
        recipient: { type: String, required: true, index: true, unique: false },
        permit: { type: permitSchema, required: false },
        amount: { type: mongoose.Schema.Types.Decimal128, required: true },
        status: { type: String, required: true, index: true },
    },
    { collection: process.env.MONGODB_COLLECTION },
);

export const ClaimModel = mongoose.model<ClaimDocument>(MONGODB_COLLECTION, claimSchema);
