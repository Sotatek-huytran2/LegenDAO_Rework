import { AzureFunction, Context, HttpRequest } from "@azure/functions";

import Attributes from "./attributes.json";

// *************** ENVIRONMENT VARIABLES  ********** //

// *************** MAIN ********** //

// the user calls this function and submits his claims (permit + address)
const HttpTrigger: AzureFunction = async function (context: Context, req: HttpRequest): Promise<void> {

    let token_id = Number(req.query["token"]);

    if (token_id < 0 || token_id > 4321) {
        context.res = {
            status: 400,
            body: "Invalid token id"
        }
        return;
    }

    let stats = Attributes.find(attr => Number(attr.id) === token_id);
    let score = { total: stats.rarity_score, rank: stats.rank };

    context.res = {
        status: 200,
        body: {
            id: token_id,
            score
        }
    }
};

export default HttpTrigger;
