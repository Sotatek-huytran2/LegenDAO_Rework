import { AzureFunction, Context, HttpRequest } from "@azure/functions";
import mongoose from "mongoose";
import { CreationModel } from "./CreationFormModel";

import { MONGODB_NAME, MONGODB_URL } from "../shared/consts";

// *************** HELPER FUNCTIONS  ********** //

interface CreationForm {
    name: string;
    email: string;
    title: string;
    details: string;
}

// *************** MAIN ********** //

// the user calls this function and submits his claims (permit + address)
const HttpTrigger: AzureFunction = async function (
    context: Context,
    req: HttpRequest,
): Promise<void> {
    await mongoose.connect(MONGODB_URL, {
        dbName: MONGODB_NAME,
        maxPoolSize: 10,
        // user: config.dbUser,
        // pass: config.dbPass,
    });

    const form: CreationForm = req.body;

    // todo: validate form

    let creationDoc = new CreationModel({
        ...form,
    });

    try {
        await creationDoc.save();
    } catch (e) {
        context.log(`failed to save creation doc: ${e}`);
        context.res = {
            status: 500,
        };
        return;
    }

    context.res = {
        status: 200,
    };

    context.log(`create doc ran successfully`);

    // await cleanup();
};

export default HttpTrigger;
