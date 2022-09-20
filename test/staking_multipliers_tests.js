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
    InstantiateWithHeight,
} = require("./cli");

const {
    addMinters,
    setTokenAttributes,
    viewTokens,
    changeWhitelistLevel,
    setViewingKey,
    addReceivingContracts,
    wrapNative,
    transferSnip721, querySubscribers, requireNotAnAdmin, waitForHeight,
} = require("./utils");

const {
    sendToMinterFromPlatform, sendToStakingFromPlatform, queryBalanceInPlatform, depositToPlatform,
} = require("./platform_utils")

const {
    queryStakingBalance,
    queryStakingRewards, withdrawFromStaking, lockToken, requireNotOwned, unlockToken, queryIsTokenLocked,
    requireLocked, requireUnlocked, requireLockedForTransfer, addStakingContractAsSubscriberToNftLock,
    addMultiplierContracts, queryBoosterItems, assertRewards,
} = require("./staking_utils")

// eslint-disable-next-line no-undef
console.warn = () => {
};

const TOKEN_PRICE = 100000000;
const TOKEN_WHITELIST_PRICE = 100000;
const REWARD_PER_BLOCK = 100;

const BOB_KEY_ON_NFT = "b-vk-n";
const BOB_KEY_ON_STAKING = "b-vk-s";
const BOB_KEY_ON_PLATFORM = "b-vk-p";
const BOB_EXTRA_LGND_AMOUNT = 5000000;
const BOB_DEPOSIT_AMOUNT = TOKEN_PRICE * 2 + BOB_EXTRA_LGND_AMOUNT;

const ALICE_KEY_ON_NFT = "a-vk-n";
const ALICE_KEY_ON_STAKING = "a-vk-s";
const ALICE_KEY_ON_PLATFORM = "a-vk-p";
const ALICE_EXTRA_LGND_AMOUNT = 8888888;
const ALICE_DEPOSIT_AMOUNT = TOKEN_PRICE + ALICE_EXTRA_LGND_AMOUNT;

// const TOTAL_DEPOSIT = BOB_DEPOSIT_AMOUNT + ALICE_DEPOSIT_AMOUNT;
const PLATFORM_KEY_ON_LGND = "p-vk-l";

const PATHS = {
    minter: "artifacts/contracts/minter_contract.wasm",
    lgnd: "artifacts/contracts/snip20.wasm",
    nft: "artifacts/contracts/snip721_lockable.wasm",
    platform: "artifacts/contracts/platform.wasm",
    staking: "artifacts/contracts/staking.wasm",
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

    const clientCharlie = await createCli(
        polarConfig.accounts[2].mnemonic,
        REST_ENDPOINT,
        chainId,
    );

    await Promise.all([
        fillUpFromFaucet(clientBob, 10_000_000),
        fillUpFromFaucet(clientAlice, 5_000_000),
        fillUpFromFaucet(clientCharlie, 5_000_000),
    ]);

    let lgndCode, lgndHash;
    let platformCode, platformHash;
    await Promise.all([
        storeCode(PATHS.lgnd, clientBob).then(result => [lgndCode, lgndHash] = result),
        storeCode(PATHS.platform, clientAlice).then(result => [platformCode, platformHash] = result),
    ]);

    let nftCode, nftHash = undefined;
    let mintCode, mintHash = undefined;
    let stakingCode, stakingHash = undefined;
    await Promise.all([
        storeCode(PATHS.nft, clientBob).then(result => [nftCode, nftHash] = result),
        storeCode(PATHS.minter, clientAlice).then(result => [mintCode, mintHash] = result),
        storeCode(PATHS.staking, clientCharlie).then(result => [stakingCode, stakingHash] = result),
    ]);

    const nftInitMsg = {
        name: "CryptidsTest",
        entropy: "YWE",
        symbol: "CRYPT",
        royalty_info: {
            decimal_places_in_rates: 3,
            royalties: [{recipient: clientBob.address, rate: 50}],
        },
    };
    const nftAddress = await Instantiate(
        clientBob,
        nftInitMsg,
        nftCode,
        nftHash,
    );

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
    const lgndAddress = await Instantiate(clientBob, lgndInitMsg, lgndCode, lgndHash);

    // wrapping uscrt instead of IBC denom because it is simpler to supply, but they should work the same
    const initialLgndBalance = "10000000000";
    await wrapNative(clientBob, lgndAddress, initialLgndBalance, "uscrt", lgndHash);
    await wrapNative(clientAlice, lgndAddress, initialLgndBalance, "uscrt", lgndHash);

    const platformInitMsg = {
        token: {
            address: lgndAddress,
            hash: lgndHash,
        },
        viewing_key: PLATFORM_KEY_ON_LGND,
        token_native_denom: "uscrt",
    };
    const platformAddress = await Instantiate(
        clientBob,
        platformInitMsg,
        platformCode,
        platformHash,
    );

    // todo is this needed?
    // await Promise.all([
    //     setViewingKey(clientBob, lgndAddress, lgndHash, BOB_KEY_ON_LGND),
    //     setViewingKey(clientAlice, lgndAddress, lgndHash, ALICE_KEY_ON_LGND),
    // ]);

    await Promise.all([
        setViewingKey(clientBob, platformAddress, platformHash, BOB_KEY_ON_PLATFORM),
        setViewingKey(clientAlice, platformAddress, platformHash, ALICE_KEY_ON_PLATFORM),
    ]);

    await Promise.all([
        depositToPlatform(clientBob, platformAddress, lgndAddress, BOB_DEPOSIT_AMOUNT),
        depositToPlatform(clientAlice, platformAddress, lgndAddress, ALICE_DEPOSIT_AMOUNT),
    ]);

    const mintingContractInitMsg = {
        nft_count: 5,
        nft_contract: {address: nftAddress, hash: nftHash},
        price: [
            {
                token: {
                    snip20: {address: lgndAddress, hash: lgndHash},
                },
                price: TOKEN_PRICE.toString(),
                whitelist_price: TOKEN_WHITELIST_PRICE.toString(),
            },
        ],
        random_seed: "YWE",
        platform: {address: platformAddress, hash: platformHash},
        only_platform: true,
    };
    const mintingContractAddress = await Instantiate(
        clientBob,
        mintingContractInitMsg,
        mintCode,
        mintHash,
    );

    await addMinters(clientBob, nftAddress, [
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
                        {trait_type: "multiplier", value: `1${i}0000`}, // nft 1 - 110%, nft 2 - 120%, and so on
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
    await addReceivingContracts(clientBob, platformAddress, [
        mintingContractAddress,
    ]);

    await sendToMinterFromPlatform(
        clientBob,
        platformAddress,
        mintingContractAddress,
        (TOKEN_PRICE * 2).toString(),
        2,
    );

    await sendToMinterFromPlatform(
        clientAlice,
        platformAddress,
        mintingContractAddress,
        (TOKEN_PRICE).toString(),
        1,
    );

    await setViewingKey(clientBob, nftAddress, nftHash, BOB_KEY_ON_NFT);
    const tokens = await viewTokens(
        clientBob,
        nftAddress,
        nftHash,
        clientBob.address,
        BOB_KEY_ON_NFT,
    );

    console.log("bob's tokens:", tokens);
    assert(tokens.length === 2, "Bob should have 2 minted tokens");

    await setViewingKey(clientAlice, nftAddress, nftHash, ALICE_KEY_ON_NFT);
    const aliceTokens = await viewTokens(
        clientAlice,
        nftAddress,
        nftHash,
        clientAlice.address,
        ALICE_KEY_ON_NFT,
    );

    console.log("alice's tokens:", aliceTokens);
    assert(aliceTokens.length === 1, "Alice should have 1 minted token");

    // try to lock token owned by someone else:
    await requireNotOwned(lockToken(clientAlice, nftAddress, nftHash, "1"));

    const stakingInitMsg = {
        token: {
            address: lgndAddress,
            hash: lgndHash,
        },
        platform: {
            address: platformAddress,
            hash: platformHash,
        },
        inflation_schedule: [
            {
                end_block: 400_000_000,
                reward_per_block: REWARD_PER_BLOCK.toString(),
            }
        ],
        viewing_key: "staking-vk",
        prng_seed: "YWE",
        // multiplier_contracts: [nftAddress],
    };
    // console.log("staking init message:", stakingInitMsg);
    const [stakingAddress, inflationStartHeight] = await InstantiateWithHeight(
        clientBob,
        stakingInitMsg,
        stakingCode,
        stakingHash,
    );
    console.log("staking contract address:", stakingAddress);
    console.log("inflationStartHeight:", inflationStartHeight);

    await Promise.all([
        setViewingKey(clientBob, stakingAddress, stakingHash, BOB_KEY_ON_STAKING),
        setViewingKey(clientAlice, stakingAddress, stakingHash, ALICE_KEY_ON_STAKING),
    ]);

    await addReceivingContracts(clientBob, platformAddress, [
        stakingAddress,
    ]);

    let bobPlatformBalance = await queryBalanceInPlatform(
        clientBob,
        platformAddress,
        platformHash,
        BOB_KEY_ON_PLATFORM,
    );
    // console.log("bob's platform balance:", bobPlatformBalance);
    assert(bobPlatformBalance.balance.staked === BOB_EXTRA_LGND_AMOUNT.toString());
    assert(bobPlatformBalance.balance.pending_redeem.unbondings.length === 0);

    console.log("sending to bob's staking");
    let height = (await sendToStakingFromPlatform(
        clientBob,
        platformAddress,
        stakingAddress,
        BOB_EXTRA_LGND_AMOUNT
    )).height;
    console.log("bob started staking on height", height);

    let bobStakingBalance = await queryStakingBalance(
        clientBob,
        stakingAddress,
        stakingHash,
        BOB_KEY_ON_STAKING,
    );
    assert(bobStakingBalance.balance.amount === BOB_EXTRA_LGND_AMOUNT.toString());
    assert(bobStakingBalance.balance.total_multiplier === "100000");
    assert(bobStakingBalance.balance.effective_multiplier === "100000");

    let bobBoosterItems = await queryBoosterItems(
        clientBob,
        stakingAddress,
        stakingHash,
        BOB_KEY_ON_STAKING,
    );
    console.log("bob's booster items:", bobBoosterItems);
    assert(bobBoosterItems.booster_items.items.length === 0);

    bobPlatformBalance = await queryBalanceInPlatform(
        clientBob,
        platformAddress,
        platformHash,
        BOB_KEY_ON_PLATFORM,
    );
    // console.log("bob's platform balance:", bobPlatformBalance);
    assert(bobPlatformBalance.balance.staked === "0");
    assert(bobPlatformBalance.balance.pending_redeem.unbondings.length === 0);

    // check rewards after 12 blocks as an example
    console.log("checking bob's staking rewards");
    let bobStakingRewards = await queryStakingRewards(
        clientBob,
        stakingAddress,
        stakingHash,
        BOB_KEY_ON_STAKING,
        inflationStartHeight + 12
    );
    assertRewards(bobStakingRewards, 12 * REWARD_PER_BLOCK, "bob")

    const bobToken = "3";
    console.log("locking one token for bob, should do nothing since the staking contract is not yet subscribed");

    await lockToken(clientBob, nftAddress, nftHash, bobToken);
    await requireLocked(queryIsTokenLocked(clientBob, nftAddress, nftHash, bobToken, BOB_KEY_ON_NFT));
    await requireLockedForTransfer(transferSnip721(clientBob, nftAddress, nftHash, clientAlice.address, bobToken));

    // check that multiplier was not added (not yet subscribed)
    bobStakingBalance = await queryStakingBalance(
        clientBob,
        stakingAddress,
        stakingHash,
        BOB_KEY_ON_STAKING,
    );
    assert(bobStakingBalance.balance.amount === BOB_EXTRA_LGND_AMOUNT.toString());
    assert(bobStakingBalance.balance.total_multiplier === "100000");

    await unlockToken(clientBob, nftAddress, nftHash, bobToken);
    await requireUnlocked(queryIsTokenLocked(clientBob, nftAddress, nftHash, bobToken, BOB_KEY_ON_NFT));

    // withdraw bob's rewards and stake for alice at the same block
    await withdrawFromStaking(clientBob, stakingAddress, stakingHash, 0); // does not wait for commit
    console.log("sending to Alice's staking");
    height = (await sendToStakingFromPlatform(
        clientAlice,
        platformAddress,
        stakingAddress,
        ALICE_EXTRA_LGND_AMOUNT
    )).height;
    console.log("alice started staking on height", height);

    // check alice's balance is reflected on staking contract
    let aliceStakingBalance = await queryStakingBalance(
        clientAlice,
        stakingAddress,
        stakingHash,
        ALICE_KEY_ON_STAKING,
    );
    assert(aliceStakingBalance.balance.amount === ALICE_EXTRA_LGND_AMOUNT.toString());
    assert(aliceStakingBalance.balance.total_multiplier === "100000");
    assert(aliceStakingBalance.balance.effective_multiplier === "100000");

    // check bob's withdrawn rewards are present in the platform
    bobPlatformBalance = await queryBalanceInPlatform(
        clientBob,
        platformAddress,
        platformHash,
        BOB_KEY_ON_PLATFORM,
    );
    let expectedBalance = ((height - inflationStartHeight) * REWARD_PER_BLOCK);
    assert(bobPlatformBalance.balance.staked === expectedBalance.toString());

    // check alice and bob's rewards are in the expected ratio in an arbitrary block in the future
    bobStakingRewards = await queryStakingRewards(
        clientBob,
        stakingAddress,
        stakingHash,
        BOB_KEY_ON_STAKING,
        height + 4
    );
    let bobExpectedRewards = Math.floor(
        4 * REWARD_PER_BLOCK * BOB_EXTRA_LGND_AMOUNT / (BOB_EXTRA_LGND_AMOUNT + ALICE_EXTRA_LGND_AMOUNT)
    );
    assertRewards(bobStakingRewards, bobExpectedRewards, "bob");

    let aliceStakingRewards = await queryStakingRewards(
        clientAlice,
        stakingAddress,
        stakingHash,
        ALICE_KEY_ON_STAKING,
        height + 4
    );
    let aliceExpectedRewards = Math.floor(
        4 * REWARD_PER_BLOCK * ALICE_EXTRA_LGND_AMOUNT /
        (BOB_EXTRA_LGND_AMOUNT + ALICE_EXTRA_LGND_AMOUNT)
    );
    assertRewards(aliceStakingRewards, aliceExpectedRewards, "alice");

    console.log("adding staking contract as subscriber to nft contract");
    await addStakingContractAsSubscriberToNftLock(clientBob, nftAddress, nftHash, stakingAddress, stakingHash);
    let subscribersResult = await querySubscribers(clientBob, nftAddress, nftHash)
    console.log("subscribers result:", subscribersResult);
    assert(subscribersResult.subscribers.contracts.length === 1);
    assert(subscribersResult.subscribers.contracts[0].toString() === {
        address: stakingAddress,
        hash: stakingHash,
    }.toString());
    console.log("subscriber registered on nft contract")

    // check unregistered multiplier contract
    try {
        let tx = await lockToken(clientBob, nftAddress, nftHash, "3");
        console.log("tx", tx);
        assert.fail("there should have been an error when locking token from an unregistered multiplier contract");
    } catch (e) {
        assert(e.tx.jsonLog.generic_err.msg.endsWith("is not allowed to set multipliers"));
        console.log("Correct error message on unregistered multiplier contract");
    }

    // check add subscribers as non-admin
    await requireNotAnAdmin(addMultiplierContracts(clientCharlie, stakingAddress, stakingHash, nftAddress));

    console.log("locking one token for both alice and bob, and withdrawing rewards");
    let tx = await addMultiplierContracts(clientBob, stakingAddress, stakingHash, nftAddress);
    let heightSecondWithdraw = tx.height + 2; // we will withdraw again for both bab and alice at this height
    let blocksPassedSinceWithdrawn = heightSecondWithdraw - height;

    bobStakingRewards = await queryStakingRewards(
        clientBob,
        stakingAddress,
        stakingHash,
        BOB_KEY_ON_STAKING,
        heightSecondWithdraw
    );
    bobExpectedRewards = Math.floor(
        blocksPassedSinceWithdrawn * REWARD_PER_BLOCK * BOB_EXTRA_LGND_AMOUNT /
        (BOB_EXTRA_LGND_AMOUNT + ALICE_EXTRA_LGND_AMOUNT)
    );
    console.log("expected height:", heightSecondWithdraw);
    assertRewards(bobStakingRewards, bobExpectedRewards, "bob");

    aliceStakingRewards = await queryStakingRewards(
        clientAlice,
        stakingAddress,
        stakingHash,
        ALICE_KEY_ON_STAKING,
        heightSecondWithdraw
    );
    aliceExpectedRewards = Math.floor(
        blocksPassedSinceWithdrawn * REWARD_PER_BLOCK * ALICE_EXTRA_LGND_AMOUNT /
        (BOB_EXTRA_LGND_AMOUNT + ALICE_EXTRA_LGND_AMOUNT)
    ) + 1; // leftover
    console.log(`alice's actual rewards:`, aliceStakingRewards);
    console.log(`alice's expected rewards:`, aliceExpectedRewards);
    assert(aliceStakingRewards.rewards.rewards === aliceExpectedRewards.toString());

    // apply multipliers at the same time, and withdraw all rewards up to now
    await waitForHeight(clientBob, heightSecondWithdraw - 1);
    await lockToken(clientBob, nftAddress, nftHash, "3", false); // does not wait for commit
    tx = await lockToken(clientAlice, nftAddress, nftHash, "4");
    let heightAfterMultipliers = tx.height;
    console.log("expected height for locking tokens:", heightSecondWithdraw);
    console.log("actual height for locking tokens:", heightAfterMultipliers);

    bobBoosterItems = await queryBoosterItems(
        clientBob,
        stakingAddress,
        stakingHash,
        BOB_KEY_ON_STAKING,
    );
    console.log("bob's booster items:", bobBoosterItems);
    assert(bobBoosterItems.booster_items.items.length === 1);
    assert.deepEqual(bobBoosterItems.booster_items.items[0],
        {
            multiplier: 130_000,
            contract: nftAddress,
            id: "3"
        }
    );

    let aliceBoosterItems = await queryBoosterItems(
        clientAlice,
        stakingAddress,
        stakingHash,
        ALICE_KEY_ON_STAKING,
    );
    console.log("alice's booster items:", aliceBoosterItems);
    assert(aliceBoosterItems.booster_items.items.length === 1);
    assert.deepEqual(aliceBoosterItems.booster_items.items[0],
        {
            multiplier: 140_000,
            contract: nftAddress,
            id: "4"
        }
    );

    console.log("checking balance in platform after locking tokens (should have pulled the rewards)");
    bobPlatformBalance = await queryBalanceInPlatform(
        clientBob,
        platformAddress,
        platformHash,
        BOB_KEY_ON_PLATFORM,
    );
    console.log("bob's platform balance:", bobPlatformBalance);
    expectedBalance += bobExpectedRewards;
    console.log("bob's expected platform balance:", expectedBalance);
    assert(bobPlatformBalance.balance.staked === expectedBalance.toString());

    let alicePlatformBalance = await queryBalanceInPlatform(
        clientAlice,
        platformAddress,
        platformHash,
        ALICE_KEY_ON_PLATFORM,
    );
    console.log("alice's platform balance:", alicePlatformBalance);
    console.log("alice's expected platform balance:", aliceExpectedRewards);
    assert(alicePlatformBalance.balance.staked === aliceExpectedRewards.toString());

    // check new multipliers are reflected on staking contract
    bobStakingBalance = await queryStakingBalance(
        clientBob,
        stakingAddress,
        stakingHash,
        BOB_KEY_ON_STAKING,
    );
    assert(bobStakingBalance.balance.amount === BOB_EXTRA_LGND_AMOUNT.toString());
    assert(bobStakingBalance.balance.total_multiplier === "130000");
    assert(bobStakingBalance.balance.effective_multiplier === "130000");
    console.log("bob's multipliers are reflected on staking balance")

    aliceStakingBalance = await queryStakingBalance(
        clientAlice,
        stakingAddress,
        stakingHash,
        ALICE_KEY_ON_STAKING,
    );
    console.log("alice actual staking balance:", aliceStakingBalance);
    assert(aliceStakingBalance.balance.amount === ALICE_EXTRA_LGND_AMOUNT.toString());
    assert(aliceStakingBalance.balance.total_multiplier === "140000");
    assert(aliceStakingBalance.balance.effective_multiplier === "140000");
    console.log("alice's multipliers are reflected on staking balance")

    // check rewards with multipliers after an arbitrary number of blocks
    bobStakingRewards = await queryStakingRewards(
        clientBob,
        stakingAddress,
        stakingHash,
        BOB_KEY_ON_STAKING,
        heightAfterMultipliers + 23
    );
    bobExpectedRewards = Math.floor(
        23 * REWARD_PER_BLOCK * BOB_EXTRA_LGND_AMOUNT * 1.3 /
        (BOB_EXTRA_LGND_AMOUNT * 1.3 + ALICE_EXTRA_LGND_AMOUNT * 1.4)
    );
    assertRewards(bobStakingRewards, bobExpectedRewards, "bob");

    aliceStakingRewards = await queryStakingRewards(
        clientAlice,
        stakingAddress,
        stakingHash,
        ALICE_KEY_ON_STAKING,
        heightAfterMultipliers + 23
    );
    aliceExpectedRewards = Math.floor(
        23 * REWARD_PER_BLOCK * ALICE_EXTRA_LGND_AMOUNT * 1.4 /
        (BOB_EXTRA_LGND_AMOUNT * 1.3 + ALICE_EXTRA_LGND_AMOUNT * 1.4)
    );
    assertRewards(aliceStakingRewards, aliceExpectedRewards, "alice");

    // lock a second token for bob on top of the first token, and reset rewards for alice
    await withdrawFromStaking(clientAlice, stakingAddress, stakingHash, 0);
    tx = await lockToken(clientBob, nftAddress, nftHash, "1");
    heightAfterMultipliers = tx.height;

    bobBoosterItems = await queryBoosterItems(
        clientBob,
        stakingAddress,
        stakingHash,
        BOB_KEY_ON_STAKING,
    );
    console.log("bob's booster items:", bobBoosterItems);
    assert(bobBoosterItems.booster_items.items.length === 2);
    assert.deepEqual(bobBoosterItems.booster_items.items,
        [
            {
                multiplier: 130_000,
                contract: nftAddress,
                id: "3"
            },
            {
                multiplier: 110_000,
                contract: nftAddress,
                id: "1"
            }
        ]
    );

    bobStakingBalance = await queryStakingBalance(
        clientBob,
        stakingAddress,
        stakingHash,
        BOB_KEY_ON_STAKING,
    );
    console.log("bob's actual staking balance:", bobStakingBalance);
    assert(bobStakingBalance.balance.amount === BOB_EXTRA_LGND_AMOUNT.toString());
    assert(bobStakingBalance.balance.total_multiplier === "140000");
    assert(bobStakingBalance.balance.effective_multiplier === "140000");
    console.log("bobs's second multipliers are reflected on staking balance")

    // check rewards with multipliers after an arbitrary number of blocks
    bobStakingRewards = await queryStakingRewards(
        clientBob,
        stakingAddress,
        stakingHash,
        BOB_KEY_ON_STAKING,
        heightAfterMultipliers + 11
    );
    bobExpectedRewards = Math.floor(
        11 * REWARD_PER_BLOCK * BOB_EXTRA_LGND_AMOUNT * (1.3 + 0.1) /
        (BOB_EXTRA_LGND_AMOUNT * (1.3 + 0.1) + ALICE_EXTRA_LGND_AMOUNT * 1.4)
    );
    assertRewards(bobStakingRewards, bobExpectedRewards, "bob");

    aliceStakingRewards = await queryStakingRewards(
        clientAlice,
        stakingAddress,
        stakingHash,
        ALICE_KEY_ON_STAKING,
        heightAfterMultipliers + 11
    );
    aliceExpectedRewards = Math.floor(
        11 * REWARD_PER_BLOCK * ALICE_EXTRA_LGND_AMOUNT * 1.4 /
        (BOB_EXTRA_LGND_AMOUNT * (1.3 + 0.1) + ALICE_EXTRA_LGND_AMOUNT * 1.4)
    );
    assertRewards(aliceStakingRewards, aliceExpectedRewards, "alice");

    // check unlock token, multiplier should return to the one with only token 1 applied
    tx = await unlockToken(clientBob, nftAddress, nftHash, "3");
    heightAfterMultipliers = tx.height;

    bobBoosterItems = await queryBoosterItems(
        clientBob,
        stakingAddress,
        stakingHash,
        BOB_KEY_ON_STAKING,
    );
    console.log("bob's booster items:", bobBoosterItems);
    assert(bobBoosterItems.booster_items.items.length === 1);
    assert.deepEqual(bobBoosterItems.booster_items.items,
        [{
            multiplier: 110_000,
            contract: nftAddress,
            id: "1"
        }]
    );

    bobStakingBalance = await queryStakingBalance(
        clientBob,
        stakingAddress,
        stakingHash,
        BOB_KEY_ON_STAKING,
    );
    console.log("bob's actual staking balance:", bobStakingBalance);
    assert(bobStakingBalance.balance.amount === BOB_EXTRA_LGND_AMOUNT.toString());
    assert(bobStakingBalance.balance.total_multiplier === "110000");
    assert(bobStakingBalance.balance.effective_multiplier === "110000");
    console.log("bobs's multipliers after unlocking are reflected on staking balance")
    bobStakingRewards = await queryStakingRewards(
        clientBob,
        stakingAddress,
        stakingHash,
        BOB_KEY_ON_STAKING,
        heightAfterMultipliers + 8
    );
    bobExpectedRewards = Math.floor(
        8 * REWARD_PER_BLOCK * BOB_EXTRA_LGND_AMOUNT * 1.1 /
        (BOB_EXTRA_LGND_AMOUNT * 1.1 + ALICE_EXTRA_LGND_AMOUNT * 1.4)
    );
    assertRewards(bobStakingRewards, bobExpectedRewards, "bob");
};

describe("staking multipliers", function () {
    this._timeout = 1000000000;

    it("staking multipliers work", test_suite);
});
