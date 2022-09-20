import {AzureFunction, Context, HttpRequest} from "@azure/functions";
import mongoose from "mongoose";

import {WhitelistDocument, WhitelistModel} from "../models/Whitelist";

// *************** ENVIRONMENT VARIABLES  ********** //

const MONGODB_NAME: string = process.env["MONGODB_NAME"] || "cryptids";
const MONGODB_URL: string = process.env["MONGODB_URL"] || "mongodb+srv://lgndTest:D24Y3XyIGo8B1Xz8@bridge-testnet.ekhng.mongodb.net/";


// *************** HELPER FUNCTIONS  ********** //

let dbInstance = undefined;

let getDbInstance = async function() {
    if (!dbInstance) {
        dbInstance = await mongoose.connect(MONGODB_URL, {
            dbName: MONGODB_NAME,
            maxPoolSize: 256,
            minPoolSize: 50
        });
    }
    return dbInstance;
};

// *************** MAIN ********** //

// the user calls this function and submits his claims (permit + address)
const HttpTrigger: AzureFunction = async function (context: Context, req: HttpRequest): Promise<void> {
    context.log('JavaScript HTTP trigger function processed a request.');

    await getDbInstance();

    const address = req.query["address"];

    if (!address) {
        //context.log("empty address");
        context.res = {
            status: 400,
            body: "Address is empty"
        }
        return;
    }

    let whitelist: WhitelistDocument = await WhitelistModel.findOne({address});

    if (whitelist) {
        context.res = {
            status: 200,
            body: {
                whitelist: true
            }
        }
    } else {
        context.res = {
            status: 200,
            body: {
                whitelist: false
            }
        }
    }
};

export default HttpTrigger;
