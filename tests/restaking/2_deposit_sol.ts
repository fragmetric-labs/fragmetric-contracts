import * as anchor from "@coral-xyz/anchor";
import {BN} from '@coral-xyz/anchor';
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";

describe("deposit_sol", async () => {
    const restaking = await restakingPlayground;
    const user1 = restaking.keychain.getKeypair('MOCK_USER1');
    const user2 = restaking.keychain.getKeypair('MOCK_USER2');

    step("try airdrop SOL to mock accounts", async function () {
        await Promise.all([
            restaking.tryAirdrop(user1.publicKey, 100),
            restaking.tryAirdrop(user2.publicKey, 100),
        ]);

        await restaking.sleep(1); // ...block hash not found?
    });

    step("user1 deposits SOL without metadata to mint fragSOL", async function () {
        const res0 = await restaking.runOperatorUpdatePrices();
        expect(res0.event.operatorUpdatedFundPrice.fundAccount.receiptTokenPrice.toNumber()).greaterThan(0);
        expect(res0.fragSOLFundBalance.toNumber()).greaterThan(0);

        const amount = new BN(10 * anchor.web3.LAMPORTS_PER_SOL);
        const res1 = await restaking.runUserDepositSOL(user1, amount, null);

        expect(res1.fragSOLFund.solOperationReservedAmount.toString()).eq(amount.toString());
        expect(res1.fragSOLFundBalance.toNumber()).greaterThan(amount.toNumber());
        expect(res1.fragSOLFundBalance.sub(res0.fragSOLFundBalance).toString()).eq(amount.toString());

        expect(res1.fragSOLUserFund.user.toString()).eq(user1.publicKey.toString());
        expect(res1.fragSOLUserFund.receiptTokenAmount.toString()).eq(res1.fragSOLUserTokenAccount.amount.toString());

        expect(res1.event.userDepositedSolToFund.walletProvider).null;
        expect(res1.event.userDepositedSolToFund.contributionAccrualRate).null;
        expect(res1.event.userDepositedSolToFund.userFundAccount.receiptTokenAmount.toString()).eq(res1.fragSOLUserTokenAccount.amount.toString())
        
        expect(res1.event.userUpdatedRewardPool.updatedUserRewardAccountAddresses.length).eq(1);
        const userRewardAccount1 = await restaking.getUserFragSOLRewardAccount(user1.publicKey);
        expect(userRewardAccount1.userRewardPools1[0].tokenAllocatedAmount.totalAmount.toString()).eq(amount.toString());
        expect(userRewardAccount1.userRewardPools1[1].tokenAllocatedAmount.totalAmount.toString()).eq(amount.toString());

        const res2 = await restaking.runOperatorUpdatePrices();
        expect(res2.event.operatorUpdatedFundPrice.fundAccount.receiptTokenPrice.toString()).eq((amount.div(new BN(res1.fragSOLUserTokenAccount.amount.toString())).mul(new BN(10 ** restaking.fragSOLDecimals))).toString());
        expect(res2.fragSOLFundBalance.sub(res0.fragSOLFundBalance).toString()).eq(amount.toString());
    });

    step("user2 deposits SOL with metadata to mint fragSOL", async function () {
        const res0 = await restaking.runOperatorUpdatePrices();

        const amount1 = new BN(6 * anchor.web3.LAMPORTS_PER_SOL);
        const currentTimestamp = new BN(Math.floor(Date.now() / 1000));
        const depositMetadata1 = restaking.asType<'depositMetadata'>({
            walletProvider: "BACKPACK",
            contributionAccrualRate: 130,
            expiredAt: currentTimestamp,
        });
        const res1 = await restaking.runUserDepositSOL(user2, amount1, depositMetadata1);

        expect(res1.fragSOLFundBalance.sub(res0.fragSOLFundBalance).toString()).eq(amount1.toString());
        expect(res1.event.userDepositedSolToFund.walletProvider).eq(depositMetadata1.walletProvider);
        expect(res1.event.userDepositedSolToFund.contributionAccrualRate.toString()).eq(depositMetadata1.contributionAccrualRate.toString());
        expect(res1.event.userDepositedSolToFund.userFundAccount.receiptTokenAmount.toString()).eq(res1.fragSOLUserTokenAccount.amount.toString())

        expect(res1.event.userUpdatedRewardPool.updatedUserRewardAccountAddresses.length).eq(1);
        const userRewardAccount1 = await restaking.getUserFragSOLRewardAccount(user2.publicKey);
        expect(userRewardAccount1.userRewardPools1[0].tokenAllocatedAmount.totalAmount.toString()).eq(amount1.toString());
        expect(userRewardAccount1.userRewardPools1[1].tokenAllocatedAmount.totalAmount.toString()).eq(amount1.toString());

        const amount2 = new BN(4 * anchor.web3.LAMPORTS_PER_SOL);
        const depositMetadata2 = restaking.asType<'depositMetadata'>({
            walletProvider: "FRONTPACK",
            contributionAccrualRate: 110,
            expiredAt: currentTimestamp,
        });
        const res2 = await restaking.runUserDepositSOL(user2, amount2, depositMetadata2);

        expect(res2.fragSOLFundBalance.sub(res1.fragSOLFundBalance).toString()).eq(amount2.toString());
        expect(res2.event.userDepositedSolToFund.walletProvider).eq(depositMetadata2.walletProvider);
        expect(res2.event.userDepositedSolToFund.contributionAccrualRate.toString()).eq(depositMetadata2.contributionAccrualRate.toString());
        expect(res2.event.userDepositedSolToFund.userFundAccount.receiptTokenAmount.toString()).eq(res2.fragSOLUserTokenAccount.amount.toString())

        expect(res2.event.userUpdatedRewardPool.updatedUserRewardAccountAddresses.length).eq(1);
        const userRewardAccount2 = await restaking.getUserFragSOLRewardAccount(user2.publicKey);
        expect(userRewardAccount2.userRewardPools1[0].tokenAllocatedAmount.totalAmount.toString()).eq(amount1.add(amount2).toString());
        expect(userRewardAccount2.userRewardPools1[0].tokenAllocatedAmount.numRecords).eq(1); // base pool has no custom accrual rate
        expect(userRewardAccount2.userRewardPools1[1].tokenAllocatedAmount.totalAmount.toString()).eq(amount1.add(amount2).toString());
        expect(userRewardAccount2.userRewardPools1[1].tokenAllocatedAmount.numRecords).eq(2);
    });

    step("user2 cannot cheat metadata", async function () {
        const amount1 = new BN(5 * anchor.web3.LAMPORTS_PER_SOL);
        const currentTimestamp = new BN(Math.floor(Date.now() / 1000));
        const depositMetadata1 = restaking.asType<'depositMetadata'>({
            walletProvider: "MYPACK",
            contributionAccrualRate: 200,
            expiredAt: currentTimestamp,
        });
        await expect(restaking.runUserDepositSOL(user2, amount1, depositMetadata1, user2)).rejectedWith('InvalidSignatureError');
    });

    step("signature verification has to fail when after expiration", async function () {
        const amount1 = new BN(5 * anchor.web3.LAMPORTS_PER_SOL);
        const expirationTimestamp = new BN(Math.floor(Date.now() / 1000) - 2); // expired 2 sec ago
        const depositMetadata1 = restaking.asType<'depositMetadata'>({
            walletProvider: "BACKPACK",
            contributionAccrualRate: 130,
            expiredAt: expirationTimestamp,
        });
        await expect(restaking.runUserDepositSOL(user2, amount1, depositMetadata1)).rejectedWith('6050'); // Simulation error. FundDepositMetadataSignatureExpiredError's error code is 6050
    });
});
