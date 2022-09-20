import { AzureFunction, Context, HttpRequest } from "@azure/functions";
// import fetch from 'node-fetch';
const fetch = require("node-fetch");

const IPFSGatewayTools = require('@pinata/ipfs-gateway-tools/dist/node');



// *************** ENVIRONMENT VARIABLES  ********** //

const IPFS_GATEWAY: string = process.env["IPFS_GATEWAY"] || "https://gateway.pinata.cloud";

// *************** HELPER FUNCTIONS  ********** //

// *************** MAIN ********** //

// the user calls this function and submits his claims (permit + address)
const HttpTrigger: AzureFunction = async function (context: Context, req: HttpRequest): Promise<void> {
    context.log('JavaScript HTTP trigger function processed a request.');

    const gatewayTools = new IPFSGatewayTools();
    const ipfsURI = req.query["uri"];
    context.log(`Got URI: ${ipfsURI}`);

    const gwURL = gatewayTools.convertToDesiredGateway(ipfsURI, IPFS_GATEWAY);
    context.log(`Got gateway URL: ${gwURL}`);

    const response = await fetch(gwURL);

    if (response.status !== 200) {
        context.res = {
            status: response.status
        };
        return;
    }

    const imageBuffer = Buffer.from(await response.arrayBuffer());

    context.res = {
        headers: {
            "Content-Type": "image/png"
        },
        isRaw: true,
        body: new Uint8Array(imageBuffer)
    };

    return;
};

export default HttpTrigger;
