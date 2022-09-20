import mongoose from "mongoose";

const MONGODB_COLLECTION = process.env.MONGODB_COLLECTION || "creation_forms";

export interface CreationDocument {
    name: string;
    email: string;
    title: string;
    details: string;
}

export const creationSchema = new mongoose.Schema<CreationDocument>(
    {
        name: {type: String, required: true},
        email: {type: String, required: true},
        title: {type: String, required: true},
        details: {type: String, required: false},
    },
    {collection: process.env.MONGODB_COLLECTION},
);

export const CreationModel = mongoose.model<CreationDocument>(MONGODB_COLLECTION, creationSchema);
