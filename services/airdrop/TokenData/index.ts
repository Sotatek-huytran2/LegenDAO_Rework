import {AzureFunction, Context, HttpRequest} from "@azure/functions";

// *************** ENVIRONMENT VARIABLES  ********** //

// *************** HELPER FUNCTIONS  ********** //

// *************** MAIN ********** //

// the user calls this function and submits his claims (permit + address)
const HttpTrigger: AzureFunction = async function (context: Context, req: HttpRequest): Promise<void> {
    context.log('JavaScript HTTP trigger function processed a request.');

    context.res = {
        status: 200,
        body: {
            price_usd: 6.3,
            apy: 85,
            liquidity: 50_000_000,
            daily_volume: 5_000_000
        }
    }

    context.log("JavaScript timer trigger function ran!");
};

export default HttpTrigger;
