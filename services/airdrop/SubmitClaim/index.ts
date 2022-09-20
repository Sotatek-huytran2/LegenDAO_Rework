import { AzureFunction, Context, HttpRequest } from "@azure/functions";
import mongoose, { HydratedDocument } from "mongoose";
import { Permit, validatePermit } from "secretjs";
import { ClaimDocument, ClaimModel } from "../shared/models/ClaimModel";

import {
    MONGODB_NAME,
    MONGODB_URL,
    PERMIT_CONTRACT_ADDRESS,
    ALLOWED_PERMISSION,
} from "../shared/consts";

// *************** HELPER FUNCTIONS  ********** //

interface Claim {
    address: string;
    permit: Permit;
}

interface Claims {
    claims: Claim[];
}

const VerifyClaim = (claim: Claim): boolean => {
    return validatePermit(
        claim.permit,
        claim.address,
        PERMIT_CONTRACT_ADDRESS,
        [ALLOWED_PERMISSION],
        false,
    );
};

// *************** MAIN ********** //

// the user calls this function and submits his claims (permit + address)
const HttpTrigger: AzureFunction = async function (
    context: Context,
    req: HttpRequest,
): Promise<void> {
    await mongoose.connect(MONGODB_URL, {
        dbName: MONGODB_NAME,
        maxPoolSize: 10,
    });

    let approvedClaims: HydratedDocument<ClaimDocument>[] = [];
    let errors = {
        notWhitelisted: [],
        failedVerification: [],
        serverError: [],
    };

    const params: Claims = req.body;

    for (const claim of params.claims) {
        let address = claim.address;
        let permit = claim.permit;

        // verify the signature
        if (VerifyClaim(claim)) {
            // get the airdrop amount
            try {
                let storedClaim = await ClaimModel.findOne({ address });

                // If the address is not whitelisted, ignore it
                if (!storedClaim) {
                    errors.notWhitelisted.push(address);
                    continue;
                }

                // if it was already claimed don't bother trying again
                if (storedClaim.status === "NotClaimed") {
                    storedClaim.permit = permit;
                    storedClaim.status = "Submitted";
                    approvedClaims.push(storedClaim);
                }
            } catch (e) {
                console.error(`failed to find airdrop doc for ${address}: ${e}`);
                errors.serverError.push(address);
            }
        } else {
            errors.failedVerification.push(address);
        }
    }

    let updatedFor = [];
    for (const claim of approvedClaims) {
        try {
            await claim.save();
            updatedFor.push(claim.address);
        } catch (e) {
            console.error(`failed to save airdrop doc for ${claim.address}: ${e}`);
            errors.serverError.push(claim.address);
        }
    }
    context.res = {
        status: 200,
        body: { claimed_for: updatedFor, errors },
    };

    console.log(`claim registered successfully for addresses: ${updatedFor}`);
};

export default HttpTrigger;
