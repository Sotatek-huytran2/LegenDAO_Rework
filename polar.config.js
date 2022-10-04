require("dotenv").config();

const accounts = [
    {
        name: "account_0",
        address: process.env.ACCOUNT_0_ADDR || "secret1ap26qrlp8mcq2pg6r47w43l0y8zkqm8a450s03",
        mnemonic:
            process.env.ACCOUNT_0_MNEMONIC ||
            "grant rice replace explain federal release fix clever romance raise often wild taxi quarter soccer fiber love must tape steak together observe swap guitar",
    },
    {
        name: "account_1",
        address: process.env.ACCOUNT_1_ADDR || "secret1fc3fzy78ttp0lwuujw7e52rhspxn8uj52zfyne",
        mnemonic:
            process.env.ACCOUNT_1_MNEMONIC ||
            "jelly shadow frog dirt dragon use armed praise universe win jungle close inmate rain oil canvas beauty pioneer chef soccer icon dizzy thunder meadow",
    },
    {
        name: "account_3",
        address: process.env.ACCOUNT_3_ADDR || "secret1ajz54hz8azwuy34qwy9fkjnfcrvf0dzswy0lqq",
        mnemonic:
            process.env.ACCOUNT_3_MNEMONIC ||
            "chair love bleak wonder skirt permit say assist aunt credit roast size obtain minute throw sand usual age smart exact enough room shadow charge",
    },
    {
        name: "account_4",
        address: process.env.ACCOUNT_4_ADDR || "secret1ldjxljw7v4vk6zhyduywh04hpj0jdwxsmrlatf",
        mnemonic:
            process.env.ACCOUNT_4_MNEMONIC ||
            "word twist toast cloth movie predict advance crumble escape whale sail such angry muffin balcony keen move employ cook valve hurt glimpse breeze brick",
    },
    {
        name: "huy_sota",
        address: process.env.ACCOUNT_5_ADDR || "secret1knm0adw6fk536hy24t8dk059yl0jz924uu9wqz",
        mnemonic:
            process.env.ACCOUNT_5_MNEMONIC ||
            "joy clip vital cigar snap column control cattle ocean scout world rude labor gun find drift gaze nurse canal soldier amazing wealth valid runway",
    },
    {
        name: "chi_hien_sota",
        address: process.env.ACCOUNT_6_ADDR || "secret159plf9h523pxrnpfh0sl06jskc4qqq89m37090",
        mnemonic:
            process.env.ACCOUNT_6_MNEMONIC ||
            "live engage small salt donate memory admit sauce tenant ability beyond voyage",
    }
];

let default_fees = {
    upload: {
        amount: [{ amount: "10000000", denom: "uscrt" }],
        gas: String(10000000),
    },
    init: {
        amount: [{ amount: "10000000", denom: "uscrt" }],
        gas: String(500000),
    },
    exec: {
        amount: [{ amount: "10000000", denom: "uscrt" }],
        gas: String(200000),
    },
    send: {
        amount: [{ amount: "20000", denom: "uscrt" }],
        gas: String(80000),
    },
};

let development = {
    endpoint: "http://0.0.0.0:1317",
    nodeId: "115aa0a629f5d70dd1d464bc7e42799e00f4edae",
    chainId: "secretdev-1",
    trustNode: true,
    keyringBackend: "test",
    accounts: accounts,
    types: {},
    fees: default_fees,
};

module.exports = {
    accounts: accounts,
    networks: {
        default: development,
        development: development,
        ci: {
            endpoint: "http://0.0.0.0:1317",
            nodeId: "115aa0a629f5d70dd1d464bc7e42799e00f4edae",
            chainId: "secretdev-1",
            trustNode: true,
            keyringBackend: "test",
            accounts: accounts,
            types: {},
            fees: default_fees,
        },
        // Pulsar-2 Testnet
        testnetP: {
            endpoint: "http://20.127.18.96:1317",
            nodeId: "6fb7169f7630da9468bf7cc0bcbbed1eb9ed0d7b",
            chainId: "pulsar-2",
            trustNode: true,
            keyringBackend: "test",
            accounts: accounts,
            types: {},
            fees: default_fees,
        },
        localnet: {
            endpoint: 'http://localhost:1317/',
            chainId: "secretdev-11",
            accounts,
        },
        testnet: {
            endpoint: "http://testnet.securesecrets.org:1317/",
            chainId: "pulsar-2",
            accounts: accounts,
        },
        mainnet: {
            endpoint: "https://secret-4.api.trivium.network:1317/",
            chainId: "secret-4",
            accounts: accounts,
        },
    },
    mocha: {
        timeout: 600000,
    },
    rust: {
        version: "1.65.0",
    },
};
