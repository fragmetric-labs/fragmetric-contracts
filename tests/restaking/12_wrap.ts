// @ts-ignore
import * as spl from '@solana/spl-token-3.x';
import {BN, IdlAccounts, web3} from '@coral-xyz/anchor';
import {expect} from 'chai';
import {step} from 'mocha-steps';
import {restakingPlayground} from '../restaking';
import { Restaking } from '../../target/types/restaking';
import { getLogger } from '../../tools/lib';
import { RestakingPlayground } from '../../tools/restaking/playground';

const {logger} = getLogger('reward');

describe("wrap", async function () {
    const restaking = await restakingPlayground as RestakingPlayground;
    const userA = restaking.keychain.getKeypair('MOCK_USER11');
    const userB = restaking.keychain.getKeypair('MOCK_USER12');

    step("try airdrop SOL to mock accounts", async function () {
        await Promise.all([
            restaking.tryAirdrop(userA.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(100)),
            restaking.tryAirdrop(userB.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(100)),
        ]);
    });

    const amountEach = new BN((10 ** restaking.fragSOLDecimals) * 10);
    step("userA deposit SOL to mint fragSOL and create accounts", async function () {
        const currentTimestamp = new BN(Math.floor(Date.now() / 1000));
        const depositMetadata = restaking.asType<'depositMetadata'>({
            user: userA.publicKey,
            walletProvider: "BACKPACK",
            contributionAccrualRate: 110,
            expiredAt: currentTimestamp,
        });
        await restaking.runUserDepositSOL(userA, amountEach.divn(2), null);
        await restaking.runUserDepositSOL(userA, amountEach.divn(2), depositMetadata);
    });

    step("userA wraps exact amount of fragSOL", async function () {
        const userAFragSOLBalance0 = await restaking.getUserFragSOLAccount(userA.publicKey).then(a => a.amount);
        const userARewardAccount0 = await restaking.getUserFragSOLRewardAccount(userA.publicKey);
        const fragSOLWrapAccount0 = await restaking.getFragSOLFundReceiptTokenWrapAccount().then(a => a.amount);
        const fragSOLWrapAccountRewardAccount0 = await restaking.getFragSOLFundWrapAccountRewardAccount();
        const fragSOLReward0 = await restaking.getFragSOLRewardAccount();

        expect(userAFragSOLBalance0.toString()).eq(amountEach.toString());
        expect(fragSOLWrapAccount0.toString()).eq("0");

        // wrap 10 fragSOL
        await restaking.runUserWrapReceiptToken(userA, amountEach.divn(2));
        const res1 = await restaking.runUserWrapReceiptToken(userA, amountEach.divn(2));

        expect(res1.fragSOLUserTokenAccount.amount.toString()).eq("0");
        expect(res1.fragSOLWrapAccount.amount.toString()).eq(amountEach.toString());
        expect(res1.wFragSOLUserTokenAccount.amount.toString()).eq(amountEach.toString());

        // global: no change
        expect(fragSOLReward0.rewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLReward.rewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq("0");
        expect(fragSOLReward0.rewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            res1.fragSOLReward.rewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq("0");
        expect(fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq("0");
        expect(res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.records[0].amount.sub( // bonus removed, so increase
            fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.divn(2).toString());
        expect(fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.records[1].amount.sub( // bonus removed, so decrease
            res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.records[1].amount).toString()).eq(amountEach.divn(2).toString());

        // userA: decrease
        expect(userARewardAccount0.userRewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLUserReward.userRewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.toString());
        expect(userARewardAccount0.userRewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            res1.fragSOLUserReward.userRewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.toString());
        expect(userARewardAccount0.userRewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLUserReward.userRewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.toString());
        expect(userARewardAccount0.userRewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            res1.fragSOLUserReward.userRewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.divn(2).toString());
        expect(userARewardAccount0.userRewardPools1[1].tokenAllocatedAmount.records[1].amount.sub(
            res1.fragSOLUserReward.userRewardPools1[1].tokenAllocatedAmount.records[1].amount).toString()).eq(amountEach.divn(2).toString());

        // wrap: increase
        expect(res1.fragSOLFundWrapAccountReward.userRewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.toString());
        expect(res1.fragSOLFundWrapAccountReward.userRewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.toString());
        expect(res1.fragSOLFundWrapAccountReward.userRewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.toString());
        expect(res1.fragSOLFundWrapAccountReward.userRewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.toString());
    })

    step("userA transfers to userB", async () => {
        await spl.createAssociatedTokenAccountIdempotent(
            restaking.connection,
            restaking.wallet,
            restaking.knownAddress.wFragSOLTokenMint,
            userB.publicKey,
        );
        await spl.transfer(
            restaking.connection,
            restaking.wallet,
            restaking.knownAddress.wFragSOLUserTokenAccount(userA.publicKey),
            restaking.knownAddress.wFragSOLUserTokenAccount(userB.publicKey),
            userA.publicKey,
            amountEach.toNumber(),
        )
    })

    step("userB unwraps fragSOL but still reward not activated", async () => {
        const userBWFragSOLBalance0 = await restaking.getUserWFragSOLAccount(userB.publicKey).then(a => a.amount);
        const fragSOLWrapAccount0 = await restaking.getFragSOLFundReceiptTokenWrapAccount().then(a => a.amount);
        const fragSOLWrapAccountRewardAccount0 = await restaking.getFragSOLFundWrapAccountRewardAccount();
        const fragSOLReward0 = await restaking.getFragSOLRewardAccount();

        expect(fragSOLWrapAccount0.toString()).eq(amountEach.toString());
        expect(userBWFragSOLBalance0.toString()).eq(amountEach.toString());

        // unwrap 10 fragSOL
        const res1 = await restaking.runUserUnwrapReceiptToken(userB, amountEach);

        expect(res1.fragSOLUserTokenAccount.amount.toString()).eq(amountEach.toString());
        expect(res1.fragSOLWrapAccount.amount.toString()).eq("0");
        expect(res1.wFragSOLUserTokenAccount.amount.toString()).eq("0");

        // global: decrease
        expect(fragSOLReward0.rewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLReward.rewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.toString());
        expect(fragSOLReward0.rewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            res1.fragSOLReward.rewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.toString());
        expect(fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.toString());
        expect(fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.toString());

        // wrap: decrease
        expect(fragSOLWrapAccountRewardAccount0.userRewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLFundWrapAccountReward.userRewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.toString());
        expect(fragSOLWrapAccountRewardAccount0.userRewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            res1.fragSOLFundWrapAccountReward.userRewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.toString());
        expect(fragSOLWrapAccountRewardAccount0.userRewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLFundWrapAccountReward.userRewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.toString());
        expect(fragSOLWrapAccountRewardAccount0.userRewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            res1.fragSOLFundWrapAccountReward.userRewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.toString());
    })

    step("userB wraps fragSOL and still reward not activated", async () => {
        const userBFragSOLBalance0 = await restaking.getUserFragSOLAccount(userB.publicKey).then(a => a.amount);
        const userBWFragSOLBalance0 = await restaking.getUserWFragSOLAccount(userB.publicKey).then(a => a.amount);
        const fragSOLWrapAccount0 = await restaking.getFragSOLFundReceiptTokenWrapAccount().then(a => a.amount);
        const fragSOLWrapAccountRewardAccount0 = await restaking.getFragSOLFundWrapAccountRewardAccount();
        const fragSOLReward0 = await restaking.getFragSOLRewardAccount();

        expect(userBFragSOLBalance0.toString()).eq(amountEach.toString());
        expect(fragSOLWrapAccount0.toString()).eq("0");
        expect(userBWFragSOLBalance0.toString()).eq("0");

        // wraps 5 fragSOL
        const res1 = await restaking.runUserWrapReceiptToken(userB, amountEach.divn(2));

        expect(res1.fragSOLUserTokenAccount.amount.toString()).eq(amountEach.divn(2).toString());
        expect(res1.fragSOLWrapAccount.amount.toString()).eq(amountEach.divn(2).toString());
        expect(res1.wFragSOLUserTokenAccount.amount.toString()).eq(amountEach.divn(2).toString());

        // global: increase
        expect(res1.fragSOLReward.rewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            fragSOLReward0.rewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.divn(2).toString());
        expect(res1.fragSOLReward.rewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLReward0.rewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.divn(2).toString());
        expect(res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.divn(2).toString());
        expect(res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.divn(2).toString());

        // wrap: increase
        expect(res1.fragSOLFundWrapAccountReward.userRewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.divn(2).toString());
        expect(res1.fragSOLFundWrapAccountReward.userRewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.divn(2).toString());
        expect(res1.fragSOLFundWrapAccountReward.userRewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.divn(2).toString());
        expect(res1.fragSOLFundWrapAccountReward.userRewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.divn(2).toString());
    })

    step("userB create accounts", async () => {
        const userBFragSOLBalance0 = await restaking.getUserFragSOLAccount(userB.publicKey).then(a => a.amount);
        const userBWFragSOLBalance0 = await restaking.getUserWFragSOLAccount(userB.publicKey).then(a => a.amount);
        const fragSOLWrapAccount0 = await restaking.getFragSOLFundReceiptTokenWrapAccount().then(a => a.amount);
        const fragSOLWrapAccountRewardAccount0 = await restaking.getFragSOLFundWrapAccountRewardAccount();
        const fragSOLReward0 = await restaking.getFragSOLRewardAccount();

        expect(userBFragSOLBalance0.toString()).eq(amountEach.divn(2).toString());
        expect(fragSOLWrapAccount0.toString()).eq(amountEach.divn(2).toString());
        expect(userBWFragSOLBalance0.toString()).eq(amountEach.divn(2).toString());

        // create account
        await restaking.runUserCreateOrUpdateFragSOLFundAndRewardAccount(userB);

        const userBRewardAccount1 = await restaking.getUserFragSOLRewardAccount(userB.publicKey);
        const fragSOLWrapAccountRewardAccount1 = await restaking.getFragSOLFundWrapAccountRewardAccount();
        const fragSOLReward1 = await restaking.getFragSOLRewardAccount();

        // global: increase
        expect(fragSOLReward1.rewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            fragSOLReward0.rewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.divn(2).toString());
        expect(fragSOLReward1.rewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLReward0.rewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.divn(2).toString());
        expect(fragSOLReward1.rewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.divn(2).toString());
        expect(fragSOLReward1.rewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.divn(2).toString());

        // userB: increase
        expect(userBRewardAccount1.userRewardPools1[0].tokenAllocatedAmount.totalAmount.toString()).eq(amountEach.divn(2).toString());
        expect(userBRewardAccount1.userRewardPools1[0].tokenAllocatedAmount.records[0].amount.toString()).eq(amountEach.divn(2).toString());
        expect(userBRewardAccount1.userRewardPools1[1].tokenAllocatedAmount.totalAmount.toString()).eq(amountEach.divn(2).toString());
        expect(userBRewardAccount1.userRewardPools1[1].tokenAllocatedAmount.records[0].amount.toString()).eq(amountEach.divn(2).toString());

        // wrap: increase
        expect(fragSOLWrapAccountRewardAccount1.userRewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.divn(2).toString());
        expect(fragSOLWrapAccountRewardAccount1.userRewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.divn(2).toString());
        expect(fragSOLWrapAccountRewardAccount1.userRewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.divn(2).toString());
        expect(fragSOLWrapAccountRewardAccount1.userRewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.divn(2).toString());
    })

    step("userB wraps desired amount of fragSOL", async () => {
        const userBFragSOLBalance0 = await restaking.getUserFragSOLAccount(userB.publicKey).then(a => a.amount);
        const userBWFragSOLBalance0 = await restaking.getUserWFragSOLAccount(userB.publicKey).then(a => a.amount);
        const userBRewardAccount0 = await restaking.getUserFragSOLRewardAccount(userB.publicKey);
        const fragSOLWrapAccount0 = await restaking.getFragSOLFundReceiptTokenWrapAccount().then(a => a.amount);
        const fragSOLWrapAccountRewardAccount0 = await restaking.getFragSOLFundWrapAccountRewardAccount();
        const fragSOLReward0 = await restaking.getFragSOLRewardAccount();

        expect(userBFragSOLBalance0.toString()).eq(amountEach.divn(2).toString());
        expect(fragSOLWrapAccount0.toString()).eq(amountEach.divn(2).toString());
        expect(userBWFragSOLBalance0.toString()).eq(amountEach.divn(2).toString());

        // wrap 10 fragSOL
        await restaking.runUserWrapReceiptTokenIfNeeded(userB, amountEach);
        const res1 = await restaking.runUserWrapReceiptTokenIfNeeded(userB, amountEach);

        expect(res1.fragSOLUserTokenAccount.amount.toString()).eq("0");
        expect(res1.fragSOLWrapAccount.amount.toString()).eq(amountEach.toString());
        expect(res1.wFragSOLUserTokenAccount.amount.toString()).eq(amountEach.toString());

        // global: no change
        expect(fragSOLReward0.rewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLReward.rewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq("0");
        expect(fragSOLReward0.rewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            res1.fragSOLReward.rewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq("0");
        expect(fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq("0");
        expect(res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.records[0].amount.sub( // bonus removed, so increase
            fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.divn(2).toString());
        expect(fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.records[1].amount.sub( // bonus removed, so decrease
            res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.records[1].amount).toString()).eq(amountEach.divn(2).toString());

        // userB: decrease
        expect(userBRewardAccount0.userRewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLUserReward.userRewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.divn(2).toString());
        expect(userBRewardAccount0.userRewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            res1.fragSOLUserReward.userRewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.divn(2).toString());
        expect(userBRewardAccount0.userRewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLUserReward.userRewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.divn(2).toString());
        expect(userBRewardAccount0.userRewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            res1.fragSOLUserReward.userRewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.divn(2).toString());

        // wrap: increase
        expect(res1.fragSOLFundWrapAccountReward.userRewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.divn(2).toString());
        expect(res1.fragSOLFundWrapAccountReward.userRewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.divn(2).toString());
        expect(res1.fragSOLFundWrapAccountReward.userRewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.divn(2).toString());
        expect(res1.fragSOLFundWrapAccountReward.userRewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.divn(2).toString());
    })
});
