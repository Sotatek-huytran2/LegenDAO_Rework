import { AzureFunction, Context, HttpRequest } from "@azure/functions";
import mongoose from "mongoose";
import { ClaimModel } from "../shared/models/ClaimModel";

import { MONGODB_NAME, MONGODB_URL } from "../shared/consts";

// the user calls this function and submits his claims (permit + address)
const HttpTrigger: AzureFunction = async function (
    context: Context,
    req: HttpRequest,
): Promise<void> {
    console.log(`connecting to: ${MONGODB_NAME} at ${MONGODB_URL}`);
    await mongoose.connect(MONGODB_URL, {
        dbName: MONGODB_NAME,
        maxPoolSize: 10,
    });

    const addresses = req.query["addresses"] || "";

    context.log(`CheckStatus(addresses = ${JSON.stringify(addresses)})`);

    let status = {};
    for await (const claimModel of ClaimModel.find().in("address", addresses.split(","))) {
        status[claimModel.address] = {
            status: claimModel.status,
            amount: claimModel.amount,
        };
    }

    context.res = {
        status: 200,
        body: {
            status,
        },
    };
};

export default HttpTrigger;
