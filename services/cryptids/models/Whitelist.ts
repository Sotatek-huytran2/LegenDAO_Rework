import mongoose from "mongoose";

const MONGODB_COLLECTION = process.env.MONGODB_COLLECTION || "whitelists";

export interface WhitelistDocument extends mongoose.Document {
    address: string,
}

export const whitelistSchema = new mongoose.Schema<WhitelistDocument>({
    address: { type: String, required: true, index: true, unique: true },
}, { collection: process.env.MONGODB_COLLECTION });


export const WhitelistModel = mongoose.model<WhitelistDocument>(MONGODB_COLLECTION, whitelistSchema);
