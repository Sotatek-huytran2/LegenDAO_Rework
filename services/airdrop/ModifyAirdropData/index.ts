// helper function to allow us to change data in the the DB
import { AzureFunction, Context, HttpRequest } from "@azure/functions";
import mongoose from "mongoose";
import { CLAIM_STATUS, ClaimModel, ClaimDocument } from "../shared/models/ClaimModel";

import { MONGODB_NAME, MONGODB_URL } from "../shared/consts";

const Decimal128 = mongoose.Types.Decimal128;
// This checks that the amount is an integer, and then turns it into a Decimal128
function IntegerDecimal128(number: string) {
    return new Decimal128(BigInt(number).toString());
}

// the user calls this function and submits his claims (permit + address)
const HttpTrigger: AzureFunction = async function (
    context: Context,
    req: HttpRequest,
): Promise<void> {
    await mongoose.connect(MONGODB_URL, {
        dbName: MONGODB_NAME,
        maxPoolSize: 10,
    });

    const address = req.query["address"];
    const recipient = req.query["recipient"];
    const status = req.query["status"] as CLAIM_STATUS;
    const amount = req.query["amount"];
    const removePermit = !!req.query["delete_permit"];

    let existing = await ClaimModel.findOne({
        address,
    });

    if (existing) {
        if (status) {
            existing.status = status;
        }
        if (recipient) {
            existing.recipient = recipient;
        }
        if (amount) {
            // This checks that the amount is an integer, and then turns it into a Decimal128
            existing.amount = IntegerDecimal128(amount);
        }
        if (removePermit) {
            existing.permit = undefined;
        }

        try {
            await existing.save();
            context.log(`updating claim for address: ${address}`);
        } catch (e) {
            context.log(`mongodb error on insert: ${e.message}`);
        }
    } else {
        if (!amount) {
            context.res = {
                status: 400,
                body: { error: "new claim must contain integer amount" },
            };
            return;
        }

        let doc: ClaimDocument = {
            address,
            recipient,
            status,
            amount: IntegerDecimal128(amount),
        };
        let newClaim = new ClaimModel(doc);
        try {
            await newClaim.save();
            context.log(`new claim for address: ${address}`);
            context.res = {
                status: 200,
            };
        } catch (e) {
            context.log(`mongodb error on insert: ${e.message}`);
            context.res = {
                status: 400,
                body: { error: `db error ${e.message}` },
            };
            return;
        }
    }
};

export default HttpTrigger;
