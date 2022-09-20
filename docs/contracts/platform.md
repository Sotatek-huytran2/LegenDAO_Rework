## Platform

### Transactions

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "HandleMsg",
  "anyOf": [
    {
      "description": "Withdraw funds from the platform, which will initiate an unbonding period",
      "type": "object",
      "required": [
        "redeem"
      ],
      "properties": {
        "redeem": {
          "type": "object",
          "properties": {
            "amount": {
              "description": "If not specified, use all funds",
              "anyOf": [
                {
                  "$ref": "#/definitions/Uint128"
                },
                {
                  "type": "null"
                }
              ]
            }
          }
        }
      }
    },
    {
      "description": "Manually claim funds that finished the unbonding time (i.e. don't wait for it to be claimed automatically)",
      "type": "object",
      "required": [
        "claim_redeemed"
      ],
      "properties": {
        "claim_redeemed": {
          "type": "object"
        }
      }
    },
    {
      "description": "Send tokens from platform to other contract in the LegenDAO ecosystem (e.g. NFT mint)",
      "type": "object",
      "required": [
        "send_from_platform"
      ],
      "properties": {
        "send_from_platform": {
          "type": "object",
          "required": [
            "contract_addr",
            "msg"
          ],
          "properties": {
            "amount": {
              "description": "If not specified, use all funds",
              "anyOf": [
                {
                  "$ref": "#/definitions/Uint128"
                },
                {
                  "type": "null"
                }
              ]
            },
            "contract_addr": {
              "description": "Destination contract",
              "allOf": [
                {
                  "$ref": "#/definitions/HumanAddr"
                }
              ]
            },
            "memo": {
              "description": "Probably not necessary",
              "type": [
                "string",
                "null"
              ]
            },
            "msg": {
              "description": "Wanted message to initiate at the destination contract (defined in the destination contract)",
              "allOf": [
                {
                  "$ref": "#/definitions/Binary"
                }
              ]
            }
          }
        }
      }
    }
  ],
  "definitions": {
    "Binary": {
      "description": "Binary is a wrapper around Vec<u8> to add base64 de/serialization with serde. It also adds some helper methods to help encode inline.\n\nThis is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>",
      "type": "string"
    },
    "HumanAddr": {
      "type": "string"
    },
    "Uint128": {
      "type": "string"
    }
  }
}
```

### Receive transactions

This section describes the inner messages of
the [Receive API](https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-20.md#receiver-interface).

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ReceiveMsg",
  "anyOf": [
    {
      "description": "Deposit funds in platform",
      "type": "object",
      "required": [
        "deposit"
      ],
      "properties": {
        "deposit": {
          "type": "object",
          "required": [
            "to"
          ],
          "properties": {
            "to": {
              "description": "The account for which the funds will be deposited",
              "allOf": [
                {
                  "$ref": "#/definitions/HumanAddr"
                }
              ]
            }
          }
        }
      }
    }
  ],
  "definitions": {
    "HumanAddr": {
      "type": "string"
    }
  }
}
```

### Transaction results

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "HandleAnswer",
  "anyOf": [
    {
      "type": "object",
      "required": [
        "deposit"
      ],
      "properties": {
        "deposit": {
          "type": "object",
          "required": [
            "status"
          ],
          "properties": {
            "status": {
              "$ref": "#/definitions/ResponseStatus"
            }
          }
        }
      }
    },
    {
      "type": "object",
      "required": [
        "redeem"
      ],
      "properties": {
        "redeem": {
          "type": "object",
          "required": [
            "status"
          ],
          "properties": {
            "status": {
              "$ref": "#/definitions/ResponseStatus"
            }
          }
        }
      }
    },
    {
      "type": "object",
      "required": [
        "claim_redeemed"
      ],
      "properties": {
        "claim_redeemed": {
          "type": "object",
          "required": [
            "status"
          ],
          "properties": {
            "status": {
              "$ref": "#/definitions/ResponseStatus"
            }
          }
        }
      }
    },
    {
      "type": "object",
      "required": [
        "send_from_platform"
      ],
      "properties": {
        "send_from_platform": {
          "type": "object",
          "required": [
            "status"
          ],
          "properties": {
            "status": {
              "$ref": "#/definitions/ResponseStatus"
            }
          }
        }
      }
    }
  ],
  "definitions": {
    "ResponseStatus": {
      "type": "string",
      "enum": [
        "success",
        "failure"
      ]
    }
  }
}
```

### Queries

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "QueryMsg",
  "anyOf": [
    {
      "description": "Number of withdraws pending to be claimed (both unbonding and claimable)",
      "type": "object",
      "required": [
        "num_of_pending_claims"
      ],
      "properties": {
        "num_of_pending_claims": {
          "type": "object"
        }
      }
    },
    {
      "description": "Balance of an account",
      "type": "object",
      "required": [
        "balance"
      ],
      "properties": {
        "balance": {
          "type": "object",
          "required": [
            "address",
            "key"
          ],
          "properties": {
            "address": {
              "description": "Address of the account",
              "allOf": [
                {
                  "$ref": "#/definitions/HumanAddr"
                }
              ]
            },
            "key": {
              "description": "Viewing key of the account",
              "type": "string"
            }
          }
        }
      }
    },
    {
      "description": "Permit queries. See more: [Permits API](https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-24.md)",
      "type": "object",
      "required": [
        "with_permit"
      ],
      "properties": {
        "with_permit": {
          "type": "object",
          "required": [
            "permit",
            "query"
          ],
          "properties": {
            "permit": {
              "$ref": "#/definitions/Permit"
            },
            "query": {
              "$ref": "#/definitions/QueryWithPermit"
            }
          }
        }
      }
    }
  ],
  "definitions": {
    "Binary": {
      "description": "Binary is a wrapper around Vec<u8> to add base64 de/serialization with serde. It also adds some helper methods to help encode inline.\n\nThis is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>",
      "type": "string"
    },
    "HumanAddr": {
      "type": "string"
    },
    "Permission": {
      "type": "string",
      "enum": [
        "allowance",
        "balance",
        "history",
        "owner"
      ]
    },
    "Permit": {
      "type": "object",
      "required": [
        "params",
        "signature"
      ],
      "properties": {
        "params": {
          "$ref": "#/definitions/PermitParams"
        },
        "signature": {
          "$ref": "#/definitions/PermitSignature"
        }
      }
    },
    "PermitParams": {
      "type": "object",
      "required": [
        "allowed_tokens",
        "chain_id",
        "permissions",
        "permit_name"
      ],
      "properties": {
        "allowed_tokens": {
          "type": "array",
          "items": {
            "$ref": "#/definitions/HumanAddr"
          }
        },
        "chain_id": {
          "type": "string"
        },
        "permissions": {
          "type": "array",
          "items": {
            "$ref": "#/definitions/Permission"
          }
        },
        "permit_name": {
          "type": "string"
        }
      }
    },
    "PermitSignature": {
      "type": "object",
      "required": [
        "pub_key",
        "signature"
      ],
      "properties": {
        "pub_key": {
          "$ref": "#/definitions/PubKey"
        },
        "signature": {
          "$ref": "#/definitions/Binary"
        }
      }
    },
    "PubKey": {
      "type": "object",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "description": "ignored, but must be \"tendermint/PubKeySecp256k1\" otherwise the verification will fail",
          "type": "string"
        },
        "value": {
          "description": "Secp256k1 PubKey",
          "allOf": [
            {
              "$ref": "#/definitions/Binary"
            }
          ]
        }
      }
    },
    "QueryWithPermit": {
      "anyOf": [
        {
          "description": "Balance of an account (the account that signed the permit). Same as QueryMsg::Balance",
          "type": "object",
          "required": [
            "balance"
          ],
          "properties": {
            "balance": {
              "type": "object"
            }
          }
        }
      ]
    }
  }
}

```

**Note:** The `WithPermit` query implements
the [Permits API](https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-24.md).

### Query results

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "QueryAnswer",
  "anyOf": [
    {
      "type": "object",
      "required": [
        "balance"
      ],
      "properties": {
        "balance": {
          "$ref": "#/definitions/Balances"
        }
      }
    },
    {
      "type": "object",
      "required": [
        "num_of_pending_claims"
      ],
      "properties": {
        "num_of_pending_claims": {
          "$ref": "#/definitions/Uint128"
        }
      }
    }
  ],
  "definitions": {
    "Balances": {
      "type": "object",
      "required": [
        "pending_redeem",
        "staked"
      ],
      "properties": {
        "pending_redeem": {
          "description": "Withdraw requests",
          "allOf": [
            {
              "$ref": "#/definitions/RedeemInfo"
            }
          ]
        },
        "staked": {
          "description": "Staked amount, not including unbonding (or claimable) funds",
          "type": "integer",
          "format": "uint128",
          "minimum": 0.0
        }
      }
    },
    "RedeemInfo": {
      "type": "object",
      "required": [
        "claimable",
        "unbondings"
      ],
      "properties": {
        "claimable": {
          "description": "Claimable withdraws (i.e. finished unbonding period)",
          "type": "integer",
          "format": "uint128",
          "minimum": 0.0
        },
        "unbondings": {
          "description": "Unbonding withdraws",
          "type": "array",
          "items": {
            "$ref": "#/definitions/UnbondingRecord"
          }
        }
      }
    },
    "Uint128": {
      "type": "string"
    },
    "UnbondingRecord": {
      "type": "object",
      "required": [
        "amount",
        "end_ts"
      ],
      "properties": {
        "amount": {
          "description": "Amount unbonding",
          "type": "integer",
          "format": "uint128",
          "minimum": 0.0
        },
        "end_ts": {
          "description": "Unbonding period ending timestamp (in seconds)",
          "type": "integer",
          "format": "uint64",
          "minimum": 0.0
        }
      }
    }
  }
}
```