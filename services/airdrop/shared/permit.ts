export type Permission = "owner" | "history" | "balance" | "Allowance";

export interface StdSignature {
    readonly pub_key: PubKey;
    readonly signature: string;
}

export interface PubKey {
    readonly type: string;
    readonly value: string;
}

export interface Permit {
    params: {
        permit_name: string;
        allowed_tokens: string[];
        chain_id: string;
        permissions: string[];
    };
    signature: StdSignature;
}
