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
        await restaking.getOrCreateUserFragSOLAccount(user8.publicKey);
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
        const user7TokenBalanceBefore = await restaking.getOrCreateUserFragSOLAccount(user7.publicKey).then(a => a.amount);
        const user8TokenBalanceBefore = await restaking.getOrCreateUserFragSOLAccount(user8.publicKey).then(a => a.amount);
        const user7RewardAccountBefore = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        const fragSOLRewardBefore = await restaking.getFragSOLRewardAccount();

        expect(user7TokenBalanceBefore.toString()).eq(amountDepositedEach.toString());
        expect(user8TokenBalanceBefore).eq(BigInt(0));

        printUserRewardAccount("bef user7", user7RewardAccountBefore);

        // user7 -> user8
        await restaking.runTransfer(user7, user8.publicKey, amountDepositedEach);

        const user7TokenBalanceAfter = await restaking.getOrCreateUserFragSOLAccount(user7.publicKey).then(a => a.amount);
        const user8TokenBalanceAfter = await restaking.getOrCreateUserFragSOLAccount(user8.publicKey).then(a => a.amount);
        const user7RewardAccountAfter = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        const fragSOLRewardAfter = await restaking.getFragSOLRewardAccount();

        expect(user7TokenBalanceAfter).eq(BigInt(0));
        expect(user8TokenBalanceAfter.toString()).eq(amountDepositedEach.toString());

        printUserRewardAccount("aft user7", user7RewardAccountAfter);

        // user8 doesn't have own userRewardAccount, so the global reward account's tokenAllocatedAmount should be deducted
        expect(fragSOLRewardBefore.rewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            fragSOLRewardAfter.rewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq(amountDepositedEach.toString());
        expect(fragSOLRewardBefore.rewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLRewardAfter.rewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq(amountDepositedEach.toString());
        expect(fragSOLRewardBefore.rewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            fragSOLRewardAfter.rewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq(amountDepositedEach.toString());
        expect(fragSOLRewardBefore.rewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLRewardAfter.rewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountDepositedEach.toString());
    });

    step("user8 transfers to user7 and user8 still does not have reward account", async () => {
        const user7TokenBalanceBefore = await restaking.getOrCreateUserFragSOLAccount(user7.publicKey).then(a => a.amount);
        const user8TokenBalanceBefore = await restaking.getOrCreateUserFragSOLAccount(user8.publicKey).then(a => a.amount);
        const user7RewardAccountBefore = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        const fragSOLRewardBefore = await restaking.getFragSOLRewardAccount();

        expect(user7TokenBalanceBefore).eq(BigInt(0));
        expect(user8TokenBalanceBefore.toString()).eq(amountDepositedEach.toString());

        printUserRewardAccount("bef user7", user7RewardAccountBefore);

        // user8 -> user7
        await restaking.runTransfer(user8, user7.publicKey, amountDepositedEach.divn(2));

        const user7TokenBalanceAfter = await restaking.getOrCreateUserFragSOLAccount(user7.publicKey).then(a => a.amount);
        const user8TokenBalanceAfter = await restaking.getOrCreateUserFragSOLAccount(user8.publicKey).then(a => a.amount);
        const user7RewardAccountAfter = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        const fragSOLRewardAfter = await restaking.getFragSOLRewardAccount();

        expect(user7TokenBalanceAfter.toString()).eq(amountDepositedEach.divn(2).toString());
        expect(user8TokenBalanceAfter.toString()).eq(amountDepositedEach.divn(2).toString());

        printUserRewardAccount("aft user7", user7RewardAccountAfter);

        expect(fragSOLRewardAfter.rewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            fragSOLRewardBefore.rewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq(amountDepositedEach.divn(2).toString());
        expect(fragSOLRewardAfter.rewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLRewardBefore.rewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq(amountDepositedEach.divn(2).toString());
        expect(fragSOLRewardAfter.rewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            fragSOLRewardBefore.rewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq(amountDepositedEach.divn(2).toString());
        expect(fragSOLRewardAfter.rewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLRewardBefore.rewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountDepositedEach.divn(2).toString());
    });

    step("user8 creates own reward account", async () => {
        const fragSOLRewardBefore = await restaking.getFragSOLRewardAccount();

        await restaking.runUserCreateOrUpdateFragSOLFundAndRewardAccount(user8);

        const user8RewardAccount = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user8.publicKey));
        const fragSOLRewardAfter = await restaking.getFragSOLRewardAccount();

        printUserRewardAccount("user8", user8RewardAccount);

        // user8 reward account has been created, so the global reward account's tokenAllocatedAmount and user8's reward account should be updated
        expect(fragSOLRewardAfter.rewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            fragSOLRewardBefore.rewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq(amountDepositedEach.divn(2).toString());
        expect(fragSOLRewardAfter.rewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLRewardBefore.rewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq(amountDepositedEach.divn(2).toString());
        expect(fragSOLRewardAfter.rewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            fragSOLRewardBefore.rewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq(amountDepositedEach.divn(2).toString());
        expect(fragSOLRewardAfter.rewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLRewardBefore.rewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountDepositedEach.divn(2).toString());

        expect(user8RewardAccount.userRewardPools1[0].tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.divn(2).toString());
        expect(user8RewardAccount.userRewardPools1[0].tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.divn(2).toString());
        expect(user8RewardAccount.userRewardPools1[1].tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.divn(2).toString());
        expect(user8RewardAccount.userRewardPools1[1].tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.divn(2).toString());
    });

    step("user7 transfers to user8", async () => {
        const user7TokenBalanceBefore = await restaking.getOrCreateUserFragSOLAccount(user7.publicKey).then(a => a.amount);
        const user8TokenBalanceBefore = await restaking.getOrCreateUserFragSOLAccount(user8.publicKey).then(a => a.amount);
        const user7RewardAccountBefore = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        const user8RewardAccountBefore = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user8.publicKey));
        const fragSOLRewardBefore = await restaking.getFragSOLRewardAccount();

        expect(user7TokenBalanceBefore.toString()).eq(amountDepositedEach.divn(2).toString());
        expect(user8TokenBalanceBefore.toString()).eq(amountDepositedEach.divn(2).toString());

        printUserRewardAccount("bef user7", user7RewardAccountBefore);
        printUserRewardAccount("bef user8", user8RewardAccountBefore);

        // user7 -> user8
        await restaking.runTransfer(user7, user8.publicKey, amountDepositedEach.divn(2));

        const user7TokenBalanceAfter = await restaking.getOrCreateUserFragSOLAccount(user7.publicKey).then(a => a.amount);
        const user8TokenBalanceAfter = await restaking.getOrCreateUserFragSOLAccount(user8.publicKey).then(a => a.amount);
        const user7RewardAccountAfter = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        const user8RewardAccountAfter = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user8.publicKey));
        const fragSOLRewardAfter = await restaking.getFragSOLRewardAccount();

        expect(user7TokenBalanceAfter).eq(BigInt(0));
        expect(user8TokenBalanceAfter.toString()).eq(amountDepositedEach.toString());

        printUserRewardAccount("aft user7", user7RewardAccountAfter);
        printUserRewardAccount("aft user8", user8RewardAccountAfter);

        // user8 has own userRewardAccount, so the global reward account's tokenAllocatedAmount should not be updated after user7's transfer
        expect(fragSOLRewardAfter.rewardPools1[0].tokenAllocatedAmount.totalAmount.toString()).eq(
            fragSOLRewardBefore.rewardPools1[0].tokenAllocatedAmount.totalAmount.toString());
        expect(fragSOLRewardAfter.rewardPools1[0].tokenAllocatedAmount.records[0].amount.toString()).eq(
            fragSOLRewardBefore.rewardPools1[0].tokenAllocatedAmount.records[0].amount.toString());
        expect(fragSOLRewardAfter.rewardPools1[1].tokenAllocatedAmount.totalAmount.toString()).eq(
            fragSOLRewardBefore.rewardPools1[1].tokenAllocatedAmount.totalAmount.toString());
        expect(fragSOLRewardAfter.rewardPools1[1].tokenAllocatedAmount.records[0].amount.toString()).eq(
            fragSOLRewardBefore.rewardPools1[1].tokenAllocatedAmount.records[0].amount.toString());

        expect(user7RewardAccountAfter.userRewardPools1[0].tokenAllocatedAmount.totalAmount.toString()).eq("0");
        expect(user7RewardAccountAfter.userRewardPools1[0].tokenAllocatedAmount.records[0].amount.toString()).eq("0");
        expect(user7RewardAccountAfter.userRewardPools1[1].tokenAllocatedAmount.totalAmount.toString()).eq("0");
        expect(user7RewardAccountAfter.userRewardPools1[1].tokenAllocatedAmount.records[0].amount.toString()).eq("0");

        expect(user8RewardAccountAfter.userRewardPools1[0].tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.toString());
        expect(user8RewardAccountAfter.userRewardPools1[0].tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.toString());
        expect(user8RewardAccountAfter.userRewardPools1[1].tokenAllocatedAmount.totalAmount.toString()).eq(amountDepositedEach.toString());
        expect(user8RewardAccountAfter.userRewardPools1[1].tokenAllocatedAmount.records[0].amount.toString()).eq(amountDepositedEach.toString());
    });

    step("deposit amount with bonus rate will disappear on transfer", async () => {
        const currentTimestamp = new BN(Math.floor(Date.now() / 1000));
        const depositMetadata = restaking.asType<'depositMetadata'>({
            user: user7.publicKey,
            walletProvider: "BACKPACK",
            contributionAccrualRate: 110,
            expiredAt: currentTimestamp,
        });
        await restaking.runUserDepositSOL(user7, amountDepositedEach, depositMetadata);

        const user7TokenBalanceBefore = await restaking.getOrCreateUserFragSOLAccount(user7.publicKey).then(a => a.amount);
        const user8TokenBalanceBefore = await restaking.getOrCreateUserFragSOLAccount(user8.publicKey).then(a => a.amount);
        const user7RewardAccountBefore = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        const user8RewardAccountBefore = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user8.publicKey));
        const fragSOLRewardBefore = await restaking.getFragSOLRewardAccount();

        expect(user7TokenBalanceBefore.toString()).eq(amountDepositedEach.toString());
        expect(user8TokenBalanceBefore.toString()).eq(amountDepositedEach.toString());

        printUserRewardAccount("bef user7", user7RewardAccountBefore);
        printUserRewardAccount("bef user8", user8RewardAccountBefore);

        // console.log(`bef base reward pool`, fragSOLRewardBefore.rewardPools1[0].tokenAllocatedAmount.records.slice(0, 2));
        // console.log(`bef bonus reward pool`, fragSOLRewardBefore.rewardPools1[1].tokenAllocatedAmount.records.slice(0, 2));

        // user7 -> user8
        await restaking.runTransfer(user7, user8.publicKey, amountDepositedEach);

        const user7TokenBalanceAfter = await restaking.getOrCreateUserFragSOLAccount(user7.publicKey).then(a => a.amount);
        const user8TokenBalanceAfter = await restaking.getOrCreateUserFragSOLAccount(user8.publicKey).then(a => a.amount);
        const user7RewardAccountAfter = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user7.publicKey));
        const user8RewardAccountAfter = await restaking.account.userRewardAccount.fetch(restaking.knownAddress.fragSOLUserReward(user8.publicKey));
        const fragSOLRewardAfter = await restaking.getFragSOLRewardAccount();

        expect(user7TokenBalanceAfter).eq(BigInt(0));
        expect(user8TokenBalanceAfter.toString()).eq(amountDepositedEach.muln(2).toString());

        printUserRewardAccount("aft user7", user7RewardAccountAfter);
        printUserRewardAccount("aft user8", user8RewardAccountAfter);

        // console.log(`aft base reward pool`, fragSOLRewardAfter.rewardPools1[0].tokenAllocatedAmount.records.slice(0, 2));
        // console.log(`aft bonus reward pool`, fragSOLRewardAfter.rewardPools1[1].tokenAllocatedAmount.records.slice(0, 2));

        // base pool total amount is equal
        expect(fragSOLRewardAfter.rewardPools1[0].tokenAllocatedAmount.totalAmount.toString()).eq(
            fragSOLRewardBefore.rewardPools1[0].tokenAllocatedAmount.totalAmount.toString());
        // base pool 1.0 amount is equal
        expect(fragSOLRewardAfter.rewardPools1[0].tokenAllocatedAmount.records[0].amount.toString()).eq(
            fragSOLRewardBefore.rewardPools1[0].tokenAllocatedAmount.records[0].amount.toString());
        // bonus pool total amount is equal
        expect(fragSOLRewardAfter.rewardPools1[1].tokenAllocatedAmount.totalAmount.toString()).eq(
            fragSOLRewardBefore.rewardPools1[1].tokenAllocatedAmount.totalAmount.toString());
        // bonus pool 1.0 amount increased
        expect(fragSOLRewardAfter.rewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLRewardBefore.rewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountDepositedEach.toString());
        // bonus pool 1.3 amount decreased
        expect(fragSOLRewardBefore.rewardPools1[1].tokenAllocatedAmount.records[1].amount.sub(
            fragSOLRewardAfter.rewardPools1[1].tokenAllocatedAmount.records[1].amount).toString()).eq(amountDepositedEach.toString());
    });
});
