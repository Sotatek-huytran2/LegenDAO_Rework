import {AzureFunction, Context, HttpRequest} from "@azure/functions";

import Attributes from "./attributes.json";

// *************** ENVIRONMENT VARIABLES  ********** //

// *************** HELPER FUNCTIONS  ********** //

const attributeNames = [
    "background", "bull_bottom_hand", "bull_head", "bull_earrings", "bull_eyes", "bull_top_hand",
    "bull_scars", "bull_mouth", "bull_clothes", "bull_fur", "female_handheld_left", "female_head", "female_eyes",
    "female_scars", "female_mouth", "female_earrings", "female_clothes", "female_fur", "female_handheld_right",
    "male_handheld_left", "male_head", "male_eyes", "male_scars", "male_mouth", "male_earrings", "male_clothes",
    "male_fur", "male_handheld_right"
]

// *************** MAIN ********** //

// the user calls this function and submits his claims (permit + address)
const HttpTrigger: AzureFunction = async function (context: Context, req: HttpRequest): Promise<void> {

    let fromParams = [];

    attributeNames.map((name) => {
          if (req.query[name]) {
              let attr = req.query[name];

              if (attr) {
                  let percentage = Attributes[name][attr]?.trait_rarity
                  let score = Attributes[name][attr]?.trait_rarity_score

                  if (score && percentage) {
                      fromParams.push({
                          score,
                          percentage,
                          type: name,
                          value: attr
                      });
                  }
              }
          }
    })

    context.res = {
        status: 200,
        body: {
            attributes: fromParams
        }
    }
};

export default HttpTrigger;
