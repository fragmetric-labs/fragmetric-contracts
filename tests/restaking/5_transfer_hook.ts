// @ts-ignore
import * as spl from '@solana/spl-token-3.x';
import {BN, IdlAccounts, web3} from '@coral-xyz/anchor';
import {expect} from 'chai';
import {step} from 'mocha-steps';
import {restakingPlayground} from '../restaking';
import { Restaking } from '../../target/types/restaking';
import { getLogger } from '../../tools/lib';

const {logger} = getLogger('reward');

function printUserRewardAccount(alias: string, account: IdlAccounts<Restaking>['userRewardAccount']) {
    for (let i = 0; i < account.numUserRewardPools; i++) {
        const pool = account.userRewardPools1[i];
        logger.debug(`[slot=${pool.updatedSlot.toString()}] ${alias}-pool#${pool.rewardPoolId}: allocated=${pool.tokenAllocatedAmount.totalAmount.toNumber().toLocaleString()}, contribution=${pool.contribution.toNumber().toLocaleString()}`);
    }
}

describe("transfer_hook", async function () {
    const restaking = await restakingPlayground;
    const user7 = restaking.keychain.getKeypair('MOCK_USER7');
    const user8 = restaking.keychain.getKeypair('MOCK_USER8');

    step("try airdrop SOL to mock accounts", async function () {
        await Promise.all([
            restaking.tryAirdrop(user7.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(100)),
            restaking.tryAirdrop(user8.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(100)),
        ]);

        await restaking.sleep(1); // ...block hash not found?
    });

    const amountDepositedEach = new BN((10 ** restaking.fragSOLDecimals) * 10);
    step("user7 deposit SOL to mint fragSOL and create accounts", async function () {
        await restaking.runUserDepositSOL(user7, amountDepositedEach, null);
    });

    step("transfer fails from client-side SDK when dest PDA is not created yet", async function () {
        // ref: node_modules/@solana/spl-token/lib/cjs/extensions/transferHook/seeds.js
        await expect(restaking.runTransfer(user7, user8.publicKey, amountDepositedEach)).rejectedWith(spl.TokenTransferHookAccountDataNotFound);
    });

    step("create user8 token account and transfer blocked from onchain-side for now", async function () {
        await restaking.runGetOrCreateTokenAccount(user8, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        await expect(restaking.runTransfer(user7, user8.publicKey, amountDepositedEach)).rejectedWith('TokenNotTransferableError');
    });

    step("enable transfer", async () => {
        const config = restaking.targetFragSOLFundConfiguration;

        await restaking.run({
            instructions: [
                restaking.methods.fundManagerUpdateFundStrategy(
                    config.depositEnabled,
                    config.donationEnabled,
                    config.withdrawalEnabled,
                    true, // transferEnabled
                    config.WithdrawalFeedRateBPS,
                    config.withdrawalBatchThresholdSeconds,
                ).accountsPartial({
                    receiptTokenMint: restaking.knownAddress.fragSOLTokenMint,
                }).instruction(),
            ],
            signerNames: ["FUND_MANAGER"],
            events: ["fundManagerUpdatedFund"],
        });
    });

    step("user7 transfers to user8 but user8 does not have reward account", async () => {
        let user7TokenAccount = await restaking.runGetOrCreateTokenAccount(user7, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        let user8TokenAccount = await restaking.runGetOrCreateTokenAccount(user8, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        let user7RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        let fragSOLReward = await restaking.account.rewardAccount.fetch(restaking.knownAddress.fragSOLReward);

        expect(user7TokenAccount.amount).eq(BigInt(amountDepositedEach.toString()));
        expect(user8TokenAccount.amount).eq(0n);

        printUserRewardAccount("bef user7", user7RewardAccount);

        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.toString());

        // user7 -> user8
        await restaking.runTransfer(user7, user8.publicKey, amountDepositedEach);

        user7TokenAccount = await restaking.runGetOrCreateTokenAccount(user7, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        user8TokenAccount = await restaking.runGetOrCreateTokenAccount(user8, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        user7RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        fragSOLReward = await restaking.account.rewardAccount.fetch(restaking.knownAddress.fragSOLReward);

        expect(user7TokenAccount.amount).eq(0n);
        expect(user8TokenAccount.amount).eq(BigInt(amountDepositedEach.toString()));

        printUserRewardAccount("aft user7", user7RewardAccount);

        // user8 doesn't have own userRewardAccount, so the global reward account's tokenAllocatedAmount should be updated to 0
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.totalAmount.toString()).eq("0");
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.records[0].amount.toString()).eq("0");
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.totalAmount.toString()).eq("0");
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.records[0].amount.toString()).eq("0");
    });

    step("user8 transfers to user7 and user8 still does not have reward account", async () => {
        let user7TokenAccount = await restaking.runGetOrCreateTokenAccount(user7, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        let user8TokenAccount = await restaking.runGetOrCreateTokenAccount(user8, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        let user7RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        let fragSOLReward = await restaking.account.rewardAccount.fetch(restaking.knownAddress.fragSOLReward);

        expect(user7TokenAccount.amount).eq(0n);
        expect(user8TokenAccount.amount).eq(BigInt(amountDepositedEach.toString()));

        printUserRewardAccount("bef user7", user7RewardAccount);

        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.totalAmount.toString()).eq("0");
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.records[0].amount.toString()).eq("0");
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.totalAmount.toString()).eq("0");
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.records[0].amount.toString()).eq("0");

        // user8 -> user7
        await restaking.runTransfer(user8, user7.publicKey, amountDepositedEach.divn(2));

        user7TokenAccount = await restaking.runGetOrCreateTokenAccount(user7, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        user8TokenAccount = await restaking.runGetOrCreateTokenAccount(user8, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        user7RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        fragSOLReward = await restaking.account.rewardAccount.fetch(restaking.knownAddress.fragSOLReward);

        expect(user7TokenAccount.amount).eq(BigInt(amountDepositedEach.divn(2).toString()));
        expect(user8TokenAccount.amount).eq(BigInt(amountDepositedEach.divn(2).toString()));

        printUserRewardAccount("aft user7", user7RewardAccount);

        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.divn(2).toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.divn(2).toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.divn(2).toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.divn(2).toString());
    });

    step("user8 creates own reward account", async () => {
        await restaking.runUserUpdateUserFragSOLFundAndRewardAccount(user8);

        let user7TokenAccount = await restaking.runGetOrCreateTokenAccount(user7, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        let user8TokenAccount = await restaking.runGetOrCreateTokenAccount(user8, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        let user7RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        let user8RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user8.publicKey));
        let fragSOLReward = await restaking.account.rewardAccount.fetch(restaking.knownAddress.fragSOLReward);

        expect(user7TokenAccount.amount).eq(BigInt(amountDepositedEach.divn(2).toString()));
        expect(user8TokenAccount.amount).eq(BigInt(amountDepositedEach.divn(2).toString()));

        printUserRewardAccount("user7", user7RewardAccount);
        printUserRewardAccount("user8", user8RewardAccount);

        // user8 reward account has been created, so the global reward account's tokenAllocatedAmount and user8's reward account should be updated
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.toString());

        expect(user8RewardAccount.userRewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.divn(2).toString());
        expect(user8RewardAccount.userRewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.divn(2).toString());
        expect(user8RewardAccount.userRewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.divn(2).toString());
        expect(user8RewardAccount.userRewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.divn(2).toString());
    });

    step("user7 transfers to user8", async () => {
        let user7TokenAccount = await restaking.runGetOrCreateTokenAccount(user7, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        let user8TokenAccount = await restaking.runGetOrCreateTokenAccount(user8, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        let user7RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        let user8RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user8.publicKey));
        let fragSOLReward = await restaking.account.rewardAccount.fetch(restaking.knownAddress.fragSOLReward);

        expect(user7TokenAccount.amount).eq(BigInt(amountDepositedEach.divn(2).toString()));
        expect(user8TokenAccount.amount).eq(BigInt(amountDepositedEach.divn(2).toString()));

        printUserRewardAccount("bef user7", user7RewardAccount);
        printUserRewardAccount("bef user8", user8RewardAccount);

        // user8 reward account has been created, so the global reward account's tokenAllocatedAmount should be updated
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.toString());

        // user7 -> user8
        await restaking.runTransfer(user7, user8.publicKey, amountDepositedEach.divn(2));

        user7TokenAccount = await restaking.runGetOrCreateTokenAccount(user7, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        user8TokenAccount = await restaking.runGetOrCreateTokenAccount(user8, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        user7RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        user8RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user8.publicKey));
        fragSOLReward = await restaking.account.rewardAccount.fetch(restaking.knownAddress.fragSOLReward);

        expect(user7TokenAccount.amount).eq(0n);
        expect(user8TokenAccount.amount).eq(BigInt(amountDepositedEach.toString()));

        printUserRewardAccount("aft user7", user7RewardAccount);
        printUserRewardAccount("aft user8", user8RewardAccount);

        // user8 has own userRewardAccount, so the global reward account's tokenAllocatedAmount should not be updated after user7's transfer
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.toString());
    });

    step("user8 transfers back to user7 again\n & global reward account's tokenAllocatedAmount should not be updated", async () => {
        let user7TokenAccount = await restaking.runGetOrCreateTokenAccount(user7, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        let user8TokenAccount = await restaking.runGetOrCreateTokenAccount(user8, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        let user7RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        let user8RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user8.publicKey));
        let fragSOLReward = await restaking.account.rewardAccount.fetch(restaking.knownAddress.fragSOLReward);

        expect(user7TokenAccount.amount).eq(0n);
        expect(user8TokenAccount.amount).eq(BigInt(amountDepositedEach.toString()));

        printUserRewardAccount("bef user7", user7RewardAccount);
        printUserRewardAccount("bef user8", user8RewardAccount);

        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.toString());

        // user8 -> user7
        await restaking.runTransfer(user8, user7.publicKey, amountDepositedEach);

        user7TokenAccount = await restaking.runGetOrCreateTokenAccount(user7, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        user8TokenAccount = await restaking.runGetOrCreateTokenAccount(user8, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        user7RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        user8RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user8.publicKey));
        fragSOLReward = await restaking.account.rewardAccount.fetch(restaking.knownAddress.fragSOLReward);

        expect(user7TokenAccount.amount).eq(BigInt(amountDepositedEach.toString()));
        expect(user8TokenAccount.amount).eq(0n);

        printUserRewardAccount("aft user7", user7RewardAccount);
        printUserRewardAccount("aft user8", user8RewardAccount);

        // global reward account's tokenAllocatedAmount should be not updated
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.toString());
    });

    step("deposit amount with bonus rate will disappear on transfer", async () => {
        const currentTimestamp = new BN(Math.floor(Date.now() / 1000));
        const depositMetadata = restaking.asType<'depositMetadata'>({
            user: user7.publicKey,
            walletProvider: "BACKPACK",
            contributionAccrualRate: 130,
            expiredAt: currentTimestamp,
        });
        const res1 = await restaking.runUserDepositSOL(user7, amountDepositedEach, depositMetadata);

        let user7TokenAccount = await restaking.runGetOrCreateTokenAccount(user7, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        let user8TokenAccount = await restaking.runGetOrCreateTokenAccount(user8, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        let user7RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        let user8RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user8.publicKey));
        let fragSOLReward = await restaking.account.rewardAccount.fetch(restaking.knownAddress.fragSOLReward);

        expect(user7TokenAccount.amount).eq(BigInt(amountDepositedEach.muln(2).toString()));
        expect(user8TokenAccount.amount).eq(0n);

        printUserRewardAccount("bef user7", user7RewardAccount);
        printUserRewardAccount("bef user8", user8RewardAccount);

        // console.log(`bef base reward pool`, fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base"));
        // console.log(`bef bonus reward pool`, fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus"));

        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.muln(2).toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.muln(2).toString());

        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.muln(2).toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.records[1].amount.toString()).eq(amountDepositedEach.toString());

        // user7 -> user8
        await restaking.runTransfer(user7, user8.publicKey, amountDepositedEach.muln(2));

        user7TokenAccount = await restaking.runGetOrCreateTokenAccount(user7, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        user8TokenAccount = await restaking.runGetOrCreateTokenAccount(user8, restaking.knownAddress.fragSOLTokenMint, restaking.knownAddress.token2022Program);
        user7RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        user8RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user8.publicKey));
        fragSOLReward = await restaking.account.rewardAccount.fetch(restaking.knownAddress.fragSOLReward);

        expect(user7TokenAccount.amount).eq(0n);
        expect(user8TokenAccount.amount).eq(BigInt(amountDepositedEach.muln(2).toString()));

        printUserRewardAccount("aft user7", user7RewardAccount);
        printUserRewardAccount("aft user8", user8RewardAccount);

        // console.log(`aft base reward pool`, fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base"));
        // console.log(`aft bonus reward pool`, fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus"));

        // global reward pool's totalAllocatedAmount.records[1].amount should be updated to 0 and records[0].amount should be updated to 20 sol
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.muln(2).toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "base").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.muln(2).toString());

        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.muln(2).toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.muln(2).toString());
        expect(fragSOLReward.rewardPools1.find((r) => restaking.binToString(r.name) == "bonus").tokenAllocatedAmount.records[1].amount.toString()).eq("0");
    });
});
