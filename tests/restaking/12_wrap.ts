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
        await restaking.runUserDepositSOL(userA, amountEach, null);
        await restaking.runUserDepositSOL(userA, amountEach, depositMetadata);
    });

    step("userA wraps fragSOL", async function () {
        const userAFragSOLBalance0 = await restaking.getUserFragSOLAccount(userA.publicKey).then(a => a.amount);
        const userARewardAccount0 = await restaking.getUserFragSOLRewardAccount(userA.publicKey);
        const fragSOLWrapAccount0 = await restaking.getFragSOLFundReceiptTokenWrapAccount().then(a => a.amount);
        const fragSOLWrapAccountRewardAccount0 = await restaking.getFragSOLFundWrapAccountRewardAccount();
        const fragSOLReward0 = await restaking.getFragSOLRewardAccount();

        expect(userAFragSOLBalance0.toString()).eq(amountEach.muln(2).toString());
        expect(fragSOLWrapAccount0.toString()).eq("0");

        // wrap 20 fragSOL
        await restaking.runUserWrapReceiptToken(userA, amountEach);
        const res1 = await restaking.runUserWrapReceiptToken(userA, amountEach);

        expect(res1.fragSOLUserTokenAccount.amount.toString()).eq("0", "user fragSOL account");
        expect(res1.fragSOLUserFund.receiptTokenAmount.toString()).eq("0", "user fund account");
        expect(res1.fragSOLWrapAccount.amount.toString()).eq(amountEach.muln(2).toString());
        expect(res1.wfragSOLUserTokenAccount.amount.toString()).eq(amountEach.muln(2).toString());

        // event check
        expect(res1.event.userWrappedReceiptToken.wrappedReceiptTokenAmount.toString()).eq(amountEach.toString());

        // global: no change
        expect(fragSOLReward0.rewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLReward.rewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq("0");
        expect(fragSOLReward0.rewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            res1.fragSOLReward.rewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq("0");
        expect(fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq("0");
        expect(res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.records[0].amount.sub( // bonus removed, so increase
            fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.toString());
        expect(fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.records[1].amount.sub( // bonus removed, so decrease
            res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.records[1].amount).toString()).eq(amountEach.toString());

        // userA: decrease
        expect(userARewardAccount0.userRewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLUserReward.userRewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.muln(2).toString());
        expect(userARewardAccount0.userRewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            res1.fragSOLUserReward.userRewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.muln(2).toString());
        expect(userARewardAccount0.userRewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLUserReward.userRewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.muln(2).toString());
        expect(userARewardAccount0.userRewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            res1.fragSOLUserReward.userRewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.toString());
        expect(userARewardAccount0.userRewardPools1[1].tokenAllocatedAmount.records[1].amount.sub(
            res1.fragSOLUserReward.userRewardPools1[1].tokenAllocatedAmount.records[1].amount).toString()).eq(amountEach.toString());

        // wrap: increase
        expect(res1.fragSOLFundWrapAccountReward.userRewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.muln(2).toString());
        expect(res1.fragSOLFundWrapAccountReward.userRewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.muln(2).toString());
        expect(res1.fragSOLFundWrapAccountReward.userRewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.muln(2).toString());
        expect(res1.fragSOLFundWrapAccountReward.userRewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.muln(2).toString());
    })

    step("userA unwraps fragSOL", async () => {
        const userAFragSOLBalance0 = await restaking.getUserFragSOLAccount(userA.publicKey).then(a => a.amount);
        const userAWfragSOLBalance0 = await restaking.getUserWfragSOLAccount(userA.publicKey).then(a => a.amount);
        const userARewardAccount0 = await restaking.getUserFragSOLRewardAccount(userA.publicKey);
        const fragSOLWrapAccount0 = await restaking.getFragSOLFundReceiptTokenWrapAccount().then(a => a.amount);
        const fragSOLWrapAccountRewardAccount0 = await restaking.getFragSOLFundWrapAccountRewardAccount();
        const fragSOLReward0 = await restaking.getFragSOLRewardAccount();

        expect(userAFragSOLBalance0.toString()).eq("0");
        expect(userAWfragSOLBalance0.toString()).eq(amountEach.muln(2).toString());
        expect(fragSOLWrapAccount0.toString()).eq(amountEach.muln(2).toString());

        // unwrap 10 fragSOL
        const res1 = await restaking.runUserUnwrapReceiptToken(userA, amountEach);

        expect(res1.fragSOLUserTokenAccount.amount.toString()).eq(amountEach.toString());
        expect(res1.fragSOLUserFund.receiptTokenAmount.toString()).eq(amountEach.toString());
        expect(res1.fragSOLWrapAccount.amount.toString()).eq(amountEach.toString());
        expect(res1.wfragSOLUserTokenAccount.amount.toString()).eq(amountEach.toString());

        // event check
        expect(res1.event.userUnwrappedReceiptToken.unwrappedReceiptTokenAmount.toString()).eq(amountEach.toString());

        // global: no change
        expect(fragSOLReward0.rewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLReward.rewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq("0");
        expect(fragSOLReward0.rewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            res1.fragSOLReward.rewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq("0");
        expect(fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq("0");
        expect(res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq("0");
        expect(fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.records[1].amount.sub(
            res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.records[1].amount).toString()).eq("0");

        // userA: increase
        expect(res1.fragSOLUserReward.userRewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            userARewardAccount0.userRewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.toString());
        expect(res1.fragSOLUserReward.userRewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            userARewardAccount0.userRewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.toString());
        expect(res1.fragSOLUserReward.userRewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            userARewardAccount0.userRewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq(amountEach.toString());
        expect(res1.fragSOLUserReward.userRewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            userARewardAccount0.userRewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq(amountEach.toString());

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

    step("userA transfers to userB", async () => {
        await spl.createAssociatedTokenAccountIdempotent(
            restaking.connection,
            restaking.wallet,
            restaking.knownAddress.wfragSOLTokenMint,
            userB.publicKey,
            {
                commitment: 'confirmed',
            }
        );

        const userAWfragSOLBalance0 = await restaking.getUserWfragSOLAccount(userA.publicKey).then(a => a.amount);
        const userBWfragSOLBalance0 = await restaking.getUserWfragSOLAccount(userB.publicKey).then(a => a.amount);

        expect(userAWfragSOLBalance0.toString()).eq(amountEach.toString(), 'bef user A');
        expect(userBWfragSOLBalance0.toString()).eq("0", 'bef user B');

        await spl.transfer(
            restaking.connection,
            restaking.wallet,
            restaking.knownAddress.wfragSOLUserTokenAccount(userA.publicKey),
            restaking.knownAddress.wfragSOLUserTokenAccount(userB.publicKey),
            userA,
            amountEach.toNumber(),
            [],
            {
                commitment: 'confirmed',
            }
        )

        const userAWfragSOLBalance1 = await restaking.getUserWfragSOLAccount(userA.publicKey).then(a => a.amount);
        const userBWfragSOLBalance1 = await restaking.getUserWfragSOLAccount(userB.publicKey).then(a => a.amount);

        expect(userAWfragSOLBalance1.toString()).eq("0", 'aft user A');
        expect(userBWfragSOLBalance1.toString()).eq(amountEach.toString(), 'aft user B');
    })

    step("userB unwraps fragSOL but still reward not activated", async () => {
        // const userBFragSOLBalance0 = await restaking.getUserFragSOLAccount(userB.publicKey).then(a => a.amount);
        const userBWfragSOLBalance0 = await restaking.getUserWfragSOLAccount(userB.publicKey).then(a => a.amount);
        // const userBRewardAccount0 = await restaking.getUserFragSOLRewardAccount(userB.publicKey);
        const fragSOLWrapAccount0 = await restaking.getFragSOLFundReceiptTokenWrapAccount().then(a => a.amount);
        const fragSOLWrapAccountRewardAccount0 = await restaking.getFragSOLFundWrapAccountRewardAccount();
        const fragSOLReward0 = await restaking.getFragSOLRewardAccount();

        // expect(userBFragSOLBalance0.toString()).eq("0");
        expect(userBWfragSOLBalance0.toString()).eq(amountEach.toString(), "user B's wfragSOL");
        expect(fragSOLWrapAccount0.toString()).eq(amountEach.toString(), 'fragSOL wrapped amount');

        // unwrap 10 fragSOL
        const res1 = await restaking.runUserUnwrapReceiptToken(userB, amountEach);

        expect(res1.fragSOLUserTokenAccount.amount.toString()).eq(amountEach.toString());
        // expect(res1.fragSOLUserFund.receiptTokenAmount.toString()).eq(amountEach.toString());
        expect(res1.fragSOLWrapAccount.amount.toString()).eq("0");
        expect(res1.wfragSOLUserTokenAccount.amount.toString()).eq("0");

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
        const userBWfragSOLBalance0 = await restaking.getUserWfragSOLAccount(userB.publicKey).then(a => a.amount);
        // const userBRewardAccount0 = await restaking.getUserFragSOLRewardAccount(userB.publicKey);
        const fragSOLWrapAccount0 = await restaking.getFragSOLFundReceiptTokenWrapAccount().then(a => a.amount);
        const fragSOLWrapAccountRewardAccount0 = await restaking.getFragSOLFundWrapAccountRewardAccount();
        const fragSOLReward0 = await restaking.getFragSOLRewardAccount();

        expect(userBFragSOLBalance0.toString()).eq(amountEach.toString());
        expect(fragSOLWrapAccount0.toString()).eq("0");
        expect(userBWfragSOLBalance0.toString()).eq("0");

        // wraps 5 fragSOL
        const res1 = await restaking.runUserWrapReceiptToken(userB, amountEach.divn(2));

        expect(res1.fragSOLUserTokenAccount.amount.toString()).eq(amountEach.divn(2).toString());
        // expect(res1.fragSOLUserFund.receiptTokenAmount.toString()).eq(amountEach.divn(2).toString());
        expect(res1.fragSOLWrapAccount.amount.toString()).eq(amountEach.divn(2).toString());
        expect(res1.wfragSOLUserTokenAccount.amount.toString()).eq(amountEach.divn(2).toString());

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
        const userBWfragSOLBalance0 = await restaking.getUserWfragSOLAccount(userB.publicKey).then(a => a.amount);
        // const userBRewardAccount0 = await restaking.getUserFragSOLRewardAccount(userB.publicKey);
        const fragSOLWrapAccount0 = await restaking.getFragSOLFundReceiptTokenWrapAccount().then(a => a.amount);
        const fragSOLWrapAccountRewardAccount0 = await restaking.getFragSOLFundWrapAccountRewardAccount();
        const fragSOLReward0 = await restaking.getFragSOLRewardAccount();

        expect(userBFragSOLBalance0.toString()).eq(amountEach.divn(2).toString());
        expect(fragSOLWrapAccount0.toString()).eq(amountEach.divn(2).toString());
        expect(userBWfragSOLBalance0.toString()).eq(amountEach.divn(2).toString());

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

        // wrap: no change
        expect(fragSOLWrapAccountRewardAccount1.userRewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq("0");
        expect(fragSOLWrapAccountRewardAccount1.userRewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq("0");
        expect(fragSOLWrapAccountRewardAccount1.userRewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq("0");
        expect(fragSOLWrapAccountRewardAccount1.userRewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLWrapAccountRewardAccount0.userRewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq("0");
    })

    step("userB wraps fragSOL up to target balance", async () => {
        const userBFragSOLBalance0 = await restaking.getUserFragSOLAccount(userB.publicKey).then(a => a.amount);
        const userBWfragSOLBalance0 = await restaking.getUserWfragSOLAccount(userB.publicKey).then(a => a.amount);
        const userBRewardAccount0 = await restaking.getUserFragSOLRewardAccount(userB.publicKey);
        const fragSOLWrapAccount0 = await restaking.getFragSOLFundReceiptTokenWrapAccount().then(a => a.amount);
        const fragSOLWrapAccountRewardAccount0 = await restaking.getFragSOLFundWrapAccountRewardAccount();
        const fragSOLReward0 = await restaking.getFragSOLRewardAccount();

        expect(userBFragSOLBalance0.toString()).eq(amountEach.divn(2).toString());
        expect(fragSOLWrapAccount0.toString()).eq(amountEach.divn(2).toString());
        expect(userBWfragSOLBalance0.toString()).eq(amountEach.divn(2).toString());

        // wrap until 10 fragSOL
        const res0 = await restaking.runUserWrapReceiptTokenIfNeeded(userB, amountEach);
        const res1 = await restaking.runUserWrapReceiptTokenIfNeeded(userB, amountEach);

        expect(res1.fragSOLUserTokenAccount.amount.toString()).eq("0");
        expect(res1.fragSOLWrapAccount.amount.toString()).eq(amountEach.toString());
        expect(res1.wfragSOLUserTokenAccount.amount.toString()).eq(amountEach.toString());

        // event check
        expect(res0.event.userWrappedReceiptToken.wrappedReceiptTokenAmount.toString()).eq(amountEach.divn(2).toString());
        expect(res1.event.userWrappedReceiptToken ?? null).to.be.null;

        // global: no change
        expect(fragSOLReward0.rewardPools1[0].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLReward.rewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq("0");
        expect(fragSOLReward0.rewardPools1[0].tokenAllocatedAmount.records[0].amount.sub(
            res1.fragSOLReward.rewardPools1[0].tokenAllocatedAmount.records[0].amount).toString()).eq("0");
        expect(fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.totalAmount.sub(
            res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq("0");
        expect(res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.records[0].amount.sub(
            fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.records[0].amount).toString()).eq("0");
        expect(fragSOLReward0.rewardPools1[1].tokenAllocatedAmount.records[1].amount.sub(
            res1.fragSOLReward.rewardPools1[1].tokenAllocatedAmount.records[1].amount).toString()).eq("0");

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
