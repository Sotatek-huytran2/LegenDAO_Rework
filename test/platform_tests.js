const assert = require("assert");
const polarConfig = require("../polar.config");
const {use} = require("chai");
const {polarChai} = require("secret-polar");
use(polarChai);
require("dotenv").config();

const {
    fillUpFromFaucet,
    createCli,
    storeCode,
    Instantiate,
} = require("./cli");
const {
    addMinters,
    setTokenAttributes,
    viewTokens,
    changeWhitelistLevel,
    setViewingKey,
    addReceivingContracts,
    wrapNative,
    queryLgndBalance,
    getScrtBalance,
    sleep,
    claimFromPlatform,
    queryIsPauser,
    queryStatusOfFeatures,
    addPauser,
    requireUnauthorized,
    pauseFeatures,
    requirePaused,
    unpauseFeatures,
} = require("./utils");
const {
    withdrawFromPlatform,
    depositToPlatform,
    sendToMinterFromPlatform,
    queryBalanceInPlatform,
} = require("./platform_utils")

// eslint-disable-next-line no-undef
console.warn = () => {
};

const TOKEN_PRICE = 100000000;
const TOKEN_WHITELIST_PRICE = 100000;

const BOB_KEY_ON_LGND = "b-vk-l";
const BOB_KEY_ON_PLATFORM = "b-vk-p";
const BOB_KEY_ON_NFT = "b-vk-n";
const BOB_EXTRA_LGND_AMOUNT = 5000000;
const BOB_DEPOSIT_AMOUNT = TOKEN_PRICE * 2 + BOB_EXTRA_LGND_AMOUNT;

const ALICE_KEY_ON_LGND = "a-vk-l";
const ALICE_KEY_ON_PLATFORM = "a-vk-p";
const ALICE_DEPOSIT_AMOUNT = 88888888;
const ALICE_WITHDRAW_1 = 22222;
const ALICE_WITHDRAW_2 = 1;

const TOTAL_DEPOSIT = BOB_DEPOSIT_AMOUNT + ALICE_DEPOSIT_AMOUNT;
const PLATFORM_KEY_ON_LGND = "p-vk-l";

const SECONDS_IN_DAY = 60 * 60 * 24;
const PATHS = {
    minter: "artifacts/contracts/minter_contract.wasm",
    lgnd: "artifacts/contracts/snip20.wasm",
    nft: "artifacts/contracts/snip721.wasm",
    platform: "artifacts/contracts/platform.wasm",
};


let test_suite = async () => {
    let REST_ENDPOINT = polarConfig.networks.ci.endpoint;
    let chainId = polarConfig.networks.ci.chainId;

    const clientBob = await createCli(
        polarConfig.accounts[0].mnemonic,
        REST_ENDPOINT,
        chainId,
    );

    const clientAlice = await createCli(
        polarConfig.accounts[1].mnemonic,
        REST_ENDPOINT,
        chainId,
    );

    if (chainId === "secretdev-1") {
        await Promise.all([
            fillUpFromFaucet(clientBob, 10_000_000),
            fillUpFromFaucet(clientAlice, 5_000_000),
        ]);
    }

    let lgndCode, lgndContractHash;
    let platformCode, platformContractHash;
    await Promise.all([
        storeCode(PATHS.lgnd, clientBob).then(result => [lgndCode, lgndContractHash] = result),
        storeCode(PATHS.platform, clientAlice).then(result => [platformCode, platformContractHash] = result),
    ]);

    let mintContractCode = undefined;
    let gpNftCode, gpNftHash = undefined;
    await Promise.all([
        storeCode(PATHS.minter, clientBob).then(result => [mintContractCode] = result),
        storeCode(PATHS.nft, clientAlice).then(result => [gpNftCode, gpNftHash] = result),
    ]);

    const nftInitMsg = {
        name: "CryptidsTest",
        entropy: "YWE",
        // revealer: secretNetwork.senderAddress,
        symbol: "CRYPT",
        royalty_info: {
            decimal_places_in_rates: 3,
            royalties: [{recipient: clientBob.address, rate: 50}],
        },
    };

    const nftContractAddress = await Instantiate(
        clientBob,
        nftInitMsg,
        gpNftCode,
    );

    const initialLgndBalance = 10000000000;

    const lgndInitMsg = {
        prng_seed: "YWE",
        symbol: "LGND",
        name: "legend",
        decimals: 6,
        initial_balances: [],
        config: {
            public_total_supply: true,
            enable_deposit: true,
            enable_redeem: true,
            enable_mint: true,
            enable_burn: true,
        },
        supported_denoms: ["uscrt", "ibc/WHATEVER"],
    };

    const lgndToken = await Instantiate(clientBob, lgndInitMsg, lgndCode);

    // wrapping uscrt instead of IBC denom because it is simpler to supply, but they should work the same
    await wrapNative(clientBob, lgndToken, initialLgndBalance.toString(), "uscrt", lgndContractHash);
    await wrapNative(clientAlice, lgndToken, initialLgndBalance.toString(), "uscrt", lgndContractHash);

    const now = Date.now();
    const nowSeconds = Math.round(now / 1000);
    const dayStart = nowSeconds - (nowSeconds % (60 * 60 * 24));

    // unbonding period is the amount of time from between today at 00:00 and the 5th block in the future.
    // This will cause the first bulk claim to be scheduled for that block.
    // Will fail in ci if run in the last seconds of the day
    const blockMs = parseInt(process.env.TX_CHECK_MS) || 6000;
    const blockTime = Math.round(blockMs / 1000);
    const timeUntilClaim = blockTime * 20;
    const unbondingPeriod = (Math.round(Date.now() / 1000) % SECONDS_IN_DAY) + timeUntilClaim;
    const expectedBulkTs = unbondingPeriod + dayStart;

    console.log("using blockMs:", blockMs);
    console.log("expected bulk end timestamp:", expectedBulkTs);
    console.log("which is UTC:", new Date(expectedBulkTs * 1000));
    console.log(`which is ${(expectedBulkTs - nowSeconds)} seconds from now`);
    console.log(`which is ${(expectedBulkTs - nowSeconds) / blockTime} blocks from now`);

    const platformInitMsg = {
        token: {
            address: lgndToken,
            hash: lgndContractHash,
        },
        viewing_key: PLATFORM_KEY_ON_LGND,
        token_native_denom: "uscrt",
        unbonding_period: unbondingPeriod,
    };

    const platformContract = await Instantiate(
        clientBob,
        platformInitMsg,
        platformCode,
    );

    await Promise.all([
        depositToPlatform(clientBob, platformContract, lgndToken, BOB_DEPOSIT_AMOUNT),
        depositToPlatform(clientAlice, platformContract, lgndToken, ALICE_DEPOSIT_AMOUNT),
    ]);

    assert(await queryIsPauser(clientAlice, platformContract, clientBob.address) === true);
    assert(await queryIsPauser(clientBob, platformContract, clientAlice.address) === false);
    let statuses = await queryStatusOfFeatures(
        clientAlice,
        platformContract,
        ["Redeem", "Claim", "Deposit", "SendFromPlatform"]
    );
    assert(statuses.find(f => f.feature === "Redeem").status === "NotPaused");
    assert(statuses.find(f => f.feature === "Claim").status === "NotPaused");
    assert(statuses.find(f => f.feature === "Deposit").status === "NotPaused");
    assert(statuses.find(f => f.feature === "SendFromPlatform").status === "NotPaused");

    await requireUnauthorized(pauseFeatures(clientAlice, platformContract, ["Deposit"], platformContractHash));
    await requireUnauthorized(unpauseFeatures(clientAlice, platformContract, ["Deposit"], platformContractHash));
    await requireUnauthorized(addPauser(clientAlice, platformContract, clientAlice.address, platformContractHash));

    await addPauser(clientBob, platformContract, clientAlice.address, platformContractHash);
    assert(await queryIsPauser(clientBob, platformContract, clientAlice.address) === true);

    await pauseFeatures(clientAlice, platformContract, ["Deposit"], platformContractHash);
    statuses = await queryStatusOfFeatures(
        clientAlice,
        platformContract,
        ["Redeem", "Claim", "Deposit", "SendFromPlatform"]
    );
    assert(statuses.find(f => f.feature === "Deposit").status === "Paused");
    assert(statuses.find(f => f.feature === "Redeem").status === "NotPaused");
    assert(statuses.find(f => f.feature === "Claim").status === "NotPaused");
    assert(statuses.find(f => f.feature === "SendFromPlatform").status === "NotPaused");

    await requirePaused(depositToPlatform(clientBob, platformContract, lgndToken, BOB_DEPOSIT_AMOUNT));

    await pauseFeatures(clientBob, platformContract, ["Redeem"], platformContractHash);
    statuses = await queryStatusOfFeatures(clientAlice, platformContract, ["Redeem"]);
    assert(statuses.find(f => f.feature === "Redeem").status === "Paused");
    await requirePaused(withdrawFromPlatform(clientBob, platformContract, 1, platformContractHash));

    await pauseFeatures(clientBob, platformContract, ["Claim"], platformContractHash);
    statuses = await queryStatusOfFeatures(clientAlice, platformContract, ["Claim"]);
    assert(statuses.find(f => f.feature === "Claim").status === "Paused");
    await requirePaused(claimFromPlatform(clientBob, platformContract, platformContractHash));

    await pauseFeatures(clientBob, platformContract, ["SendFromPlatform"], platformContractHash);
    statuses = await queryStatusOfFeatures(clientAlice, platformContract, ["SendFromPlatform"]);
    assert(statuses.find(f => f.feature === "SendFromPlatform").status === "Paused");
    await requirePaused(sendToMinterFromPlatform(clientBob, platformContract, clientAlice.address, 2, 2));

    await unpauseFeatures(clientBob,
        platformContract,
        ["Redeem", "Claim", "Deposit", "SendFromPlatform"],
        platformContractHash);

    let platformLgndBalance = await queryLgndBalance(clientBob,
        lgndToken,
        lgndContractHash,
        platformContract,
        PLATFORM_KEY_ON_LGND,
    );

    console.log("initial platform lgnd balance:", platformLgndBalance);
    assert(platformLgndBalance === TOTAL_DEPOSIT, "platform's LGND balance should equal deposit amount");

    await Promise.all([
        setViewingKey(clientBob, lgndToken, lgndContractHash, BOB_KEY_ON_LGND),
        setViewingKey(clientAlice, lgndToken, lgndContractHash, ALICE_KEY_ON_LGND),
    ]);

    let bobLgndBalance = await queryLgndBalance(
        clientBob,
        lgndToken,
        lgndContractHash,
        clientBob.address,
        BOB_KEY_ON_LGND,
    );
    console.log("bob's lgndBalance after depositing to platform", bobLgndBalance)
    assert(
        bobLgndBalance === initialLgndBalance - BOB_DEPOSIT_AMOUNT,
        "bob's LGND balance should be the wrapped amount minus the amount deposited on platform",
    );

    let aliceLgndBalance = await queryLgndBalance(
        clientAlice,
        lgndToken,
        lgndContractHash,
        clientAlice.address,
        ALICE_KEY_ON_LGND,
    );
    console.log("alice's lgndBalance after depositing to platform", bobLgndBalance)
    assert(
        aliceLgndBalance === initialLgndBalance - ALICE_DEPOSIT_AMOUNT,
        "alice's LGND balance should be the wrapped amount minus the amount deposited on platform",
    );

    // test wrong viewing key:
    const queryResult = await queryBalanceInPlatform(
        clientBob,
        platformContract,
        platformContractHash,
        BOB_KEY_ON_PLATFORM,
    );

    assert(queryResult.unauthorized);

    await Promise.all([
        setViewingKey(clientBob, platformContract, platformContractHash, BOB_KEY_ON_PLATFORM),
        setViewingKey(clientAlice, platformContract, platformContractHash, ALICE_KEY_ON_PLATFORM),
    ]);

    let bobBalanceInPlatform = await queryBalanceInPlatform(
        clientBob,
        platformContract,
        platformContractHash,
        BOB_KEY_ON_PLATFORM,
    );

    assert(parseInt(bobBalanceInPlatform.balance.staked) === BOB_DEPOSIT_AMOUNT);
    assert(bobBalanceInPlatform.balance.pending_redeem.claimable === "0");
    assert(bobBalanceInPlatform.balance.pending_redeem.unbondings.length === 0);

    let aliceBalanceInPlatform = await queryBalanceInPlatform(
        clientAlice,
        platformContract,
        platformContractHash,
        ALICE_KEY_ON_PLATFORM,
    );

    assert(parseInt(aliceBalanceInPlatform.balance.staked) === ALICE_DEPOSIT_AMOUNT);
    assert(aliceBalanceInPlatform.balance.pending_redeem.claimable === "0");
    assert(aliceBalanceInPlatform.balance.pending_redeem.unbondings.length === 0);

    const totalPlatformBalance = await clientBob.query.compute.queryContract({
        contractAddress: platformContract,
        codeHash: platformContractHash,
        query: {total_balances: {}}
    });
    console.log("total balance:", totalPlatformBalance);
    assert(totalPlatformBalance.total_balances.unbonding === "0");
    console.log("total staked:", parseInt(totalPlatformBalance.total_balances.staked));
    console.log("expected staked:", (TOTAL_DEPOSIT / 1000000 >> 0) * 1000000);
    assert(
        parseInt(totalPlatformBalance.total_balances.staked) ===
        (TOTAL_DEPOSIT / 1000000 >> 0) * 1000000 // obfuscates all digits but the first 3
    );

    let pendingClaimsNum = await clientBob.query.compute.queryContract({
        contractAddress: platformContract,
        codeHash: platformContractHash,
        query: {num_of_pending_claims: {}}
    });
    assert(pendingClaimsNum.num_of_pending_claims === "0");

    await withdrawFromPlatform(clientBob, platformContract, BOB_EXTRA_LGND_AMOUNT, platformContractHash);

    bobBalanceInPlatform = await queryBalanceInPlatform(
        clientBob,
        platformContract,
        platformContractHash,
        BOB_KEY_ON_PLATFORM,
    );

    console.log(
        "Bob's balance in platform after right after withdrawal request",
        JSON.stringify(bobBalanceInPlatform, null, 2)
    );
    assert(bobBalanceInPlatform.balance.staked === (BOB_DEPOSIT_AMOUNT - BOB_EXTRA_LGND_AMOUNT).toString());
    assert(bobBalanceInPlatform.balance.pending_redeem.unbondings[0].amount === BOB_EXTRA_LGND_AMOUNT.toString());
    assert(bobBalanceInPlatform.balance.pending_redeem.unbondings[0].end_ts === expectedBulkTs);

    platformLgndBalance =
        await queryLgndBalance(clientBob, lgndToken, lgndContractHash, platformContract, PLATFORM_KEY_ON_LGND);
    assert(
        platformLgndBalance === TOTAL_DEPOSIT,
        "platform's LGND balance should remain the same until withdrawal is claimed after unbonding period",
    );

    pendingClaimsNum = await clientBob.query.compute.queryContract({
        contractAddress: platformContract,
        codeHash: platformContractHash,
        query: {num_of_pending_claims: {}}
    });
    assert(pendingClaimsNum.num_of_pending_claims === "1");

    await withdrawFromPlatform(clientAlice, platformContract, ALICE_WITHDRAW_1, platformContractHash);
    await withdrawFromPlatform(clientAlice, platformContract, ALICE_WITHDRAW_2, platformContractHash);

    aliceBalanceInPlatform = await queryBalanceInPlatform(
        clientAlice,
        platformContract,
        platformContractHash,
        ALICE_KEY_ON_PLATFORM,
    );

    assert(parseInt(aliceBalanceInPlatform.balance.staked) ===
        ALICE_DEPOSIT_AMOUNT -
        ALICE_WITHDRAW_1 -
        ALICE_WITHDRAW_2);
    assert(aliceBalanceInPlatform.balance.pending_redeem.claimable === "0");
    assert(aliceBalanceInPlatform.balance.pending_redeem.unbondings.length === 1);

    // note that it does not wait here, this is just the point
    // from which to start counting the time elapsed since withdrawal
    const withdrawExiprationBulk = sleep(timeUntilClaim);

    pendingClaimsNum = await clientBob.query.compute.queryContract({
        contractAddress: platformContract,
        codeHash: platformContractHash,
        query: {num_of_pending_claims: {}}
    });
    console.log("pendingClaimsNum.num_of_pending_claims:", pendingClaimsNum.num_of_pending_claims);
    assert(
        pendingClaimsNum.num_of_pending_claims === "2",
        "Second withdraw with the same end timestamp should only update the existing claim"
    );

    const mintingContractInitMsg = {
        nft_count: 5,
        nft_contract: {address: nftContractAddress, hash: gpNftHash},
        price: [
            {
                token: {
                    snip20: {address: lgndToken, hash: lgndContractHash},
                },
                price: TOKEN_PRICE.toString(),
                whitelist_price: TOKEN_WHITELIST_PRICE.toString(),
            },
        ],
        random_seed: "YWE",
        platform: {address: platformContract, hash: platformContractHash},
        only_platform: true,
    };

    const mintingContractAddress = await Instantiate(
        clientBob,
        mintingContractInitMsg,
        mintContractCode,
    );


    await addMinters(clientBob, nftContractAddress, [
        mintingContractAddress,
    ]);

    const attributes = [];

    for (let i = 0; i < 5; i++) {
        attributes.push({
            token_id: i.toString(10),
            attributes: {
                public_attributes: {
                    custom_traits: [
                        {trait_type: "sword_size", value: `${i}`},
                    ],
                    rarity: 0,
                    token_uri:
                        "https://data.whicdn.com/images/311555755/original.jpg",
                    description: "cool desc",
                    name: "cool name",
                    external_url: "https://scrtlabs.com",
                },
                private_attributes: {
                    custom_traits: [
                        {trait_type: "shield type", value: `${i}`},
                    ],
                    rarity: 0,
                    token_uri:
                        "https://i1.sndcdn.com/artworks-000020949523-d1u8n6-t500x500.jpg",
                    description: "cool desc",
                    name: "cool name",
                    external_url: "https://scrtlabs.com",
                },
            },
        });
    }

    await setTokenAttributes(clientBob, mintingContractAddress, attributes);

    console.log(`changing whitelist level to public`);
    await changeWhitelistLevel(clientBob, mintingContractAddress, 3);

    const bobBalanceBefore = BigInt(await getScrtBalance(clientBob));
    console.log("Bob's balance before auto-claim:", bobBalanceBefore);
    const aliceBalanceBeforeBobClaim = BigInt(await getScrtBalance(clientAlice));

    console.log("waiting to make sure the time for auto-claim has arrived...");
    await withdrawExiprationBulk; // make sure the claim wait expired
    console.log("withdraw waiting expired, auto-claim should occur now");
    await addReceivingContracts(clientBob, platformContract, [
        mintingContractAddress,
    ]);

    const aliceBalanceAfterBobClaim = BigInt(await getScrtBalance(clientAlice));
    assert(aliceBalanceBeforeBobClaim === aliceBalanceAfterBobClaim);

    const bobBalanceAfterAutoClaim = BigInt(await getScrtBalance(clientBob));
    console.log("bobBalanceAfterAutoClaim", bobBalanceAfterAutoClaim);
    console.log("expected bobBalanceAfterAutoClaim", bobBalanceBefore + BigInt(BOB_EXTRA_LGND_AMOUNT) - 100_000n / 4n);
    console.log("difference = ",
        bobBalanceAfterAutoClaim - (bobBalanceBefore + BigInt(BOB_EXTRA_LGND_AMOUNT) - 100_000n / 4n));
    assert(
        bobBalanceAfterAutoClaim ===
        bobBalanceBefore + BigInt(BOB_EXTRA_LGND_AMOUNT) - 100_000n / 4n // fees paid in last execution
    );

    pendingClaimsNum = await clientBob.query.compute.queryContract({
        contractAddress: platformContract,
        codeHash: platformContractHash,
        query: {num_of_pending_claims: {}}
    });
    assert(pendingClaimsNum.num_of_pending_claims === "1", "bob's claim should be auto-claimed");

    bobLgndBalance =
        await queryLgndBalance(clientBob, lgndToken, lgndContractHash, clientBob.address, BOB_KEY_ON_LGND);
    assert(
        bobLgndBalance === initialLgndBalance - BOB_DEPOSIT_AMOUNT,
        "Although auto-claimed from platform, the snip20 balance of the claimer " +
        "should remain the same because the withdrawn coins are unwrapped"
    );

    platformLgndBalance =
        await queryLgndBalance(clientBob, lgndToken, lgndContractHash, platformContract, PLATFORM_KEY_ON_LGND);
    console.log("platform lgnd balance after auto-claim:", platformLgndBalance);
    console.log("expected", TOTAL_DEPOSIT - BOB_EXTRA_LGND_AMOUNT);
    assert(
        platformLgndBalance === TOTAL_DEPOSIT - BOB_EXTRA_LGND_AMOUNT,
        "platform LGND balance should reflect auto-claim sent to bob"
    );

    const aliceBalanceBefore = BigInt(await getScrtBalance(clientAlice));
    console.log("alice balance before auto-claim:", bobBalanceBefore);

    await sendToMinterFromPlatform(
        clientBob,
        platformContract,
        mintingContractAddress,
        (TOKEN_PRICE * 2).toString(),
        2,
    );

    const aliceBalanceAfterAutoClaim = BigInt(await getScrtBalance(clientAlice));
    const expectedBalance = aliceBalanceBefore + BigInt(ALICE_WITHDRAW_1) + BigInt(ALICE_WITHDRAW_2);

    console.log("aliceBalanceAfterAutoClaim:", aliceBalanceAfterAutoClaim);
    console.log("expected aliceBalanceAfterAutoClaim:", expectedBalance);
    assert(
        aliceBalanceAfterAutoClaim === expectedBalance
    );

    pendingClaimsNum = await clientAlice.query.compute.queryContract({
        contractAddress: platformContract,
        codeHash: platformContractHash,
        query: {num_of_pending_claims: {}}
    });
    assert(pendingClaimsNum.num_of_pending_claims === "0", "both of alice's claims should be auto-claimed together");

    platformLgndBalance =
        await queryLgndBalance(clientBob, lgndToken, lgndContractHash, platformContract, PLATFORM_KEY_ON_LGND);
    console.log("platform lgnd balance after auto-claims:", platformLgndBalance);
    console.log("expected platform lgnd balance after auto-claims:",
        TOTAL_DEPOSIT - BOB_EXTRA_LGND_AMOUNT - ALICE_WITHDRAW_1 - ALICE_WITHDRAW_2);
    assert(
        platformLgndBalance === TOTAL_DEPOSIT - BOB_DEPOSIT_AMOUNT - ALICE_WITHDRAW_1 - ALICE_WITHDRAW_2,
        "platform LGND balance should reflect auto-claim sent to alice"
    );

    try {
        await claimFromPlatform(clientBob, platformContract, platformContractHash);
        assert(false, "claiming from platform with no available claims should have raised an error");
    } catch (e) {
        assert(e.tx.jsonLog.generic_err.msg === "nothing to claim")
    }

    platformLgndBalance =
        await queryLgndBalance(clientBob, lgndToken, lgndContractHash, platformContract, PLATFORM_KEY_ON_LGND);
    console.log("platform lgnd balance after auto-claim:", platformLgndBalance);
    assert(
        platformLgndBalance === TOTAL_DEPOSIT - BOB_DEPOSIT_AMOUNT - ALICE_WITHDRAW_1 - ALICE_WITHDRAW_2,
        "platform LGND balance should reflect auto-claim sent to alice"
    );

    await setViewingKey(clientBob, nftContractAddress, gpNftHash, BOB_KEY_ON_NFT);
    const tokens = await viewTokens(
        clientBob,
        nftContractAddress,
        gpNftHash,
        clientBob.address,
        BOB_KEY_ON_NFT,
    );

    assert(tokens.length === 2, "Bob should have 2 minted tokens");

    bobBalanceInPlatform = await queryBalanceInPlatform(
        clientBob,
        platformContract,
        platformContractHash,
        BOB_KEY_ON_PLATFORM,
    );

    assert(parseInt(bobBalanceInPlatform.balance.staked) === 0);

    // try sending to a wrong contract
    try {
        await sendToMinterFromPlatform(clientBob, platformContract, clientAlice.address, 2, 2);
        assert(false, `sending to a wrong contract should have raised an error`);
    } catch (e) {
        assert(
            e.tx.jsonLog.generic_err.msg ===
            `address ${clientAlice.address} is not a receiving contract, sending tokens from platform is not allowed`
        );
    }
};

describe("platform", function () {
    this._timeout = 1000000000;

    it("platform works", test_suite);
});
