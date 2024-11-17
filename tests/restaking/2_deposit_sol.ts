import * as anchor from "@coral-xyz/anchor";
import {BN} from "@coral-xyz/anchor";
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";

module.exports = (i: number) => describe(`deposit_sol#${i}`, async () => {
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
        expect(res0.event.operatorUpdatedFundPrice.fundAccount.oneReceiptTokenAsSol.toNumber())
            .greaterThan(0, 'fragSOL price is not zero');
        expect(res0.fragSOLFund.solOperationReservedAmount.add(res0.fragSOLFund.withdrawal.solWithdrawalReservedAmount).toNumber())
            .eq(res0.fragSOLFundReserveAccountBalance.toNumber(), 'fund records correct amount of SOL reserved');
        const userFundAccount0 = await restaking.getUserFragSOLFundAccount(user1.publicKey).catch(v => null);
        const userRewardAccount0 = await restaking.getUserFragSOLRewardAccount(user1.publicKey).catch(v => null);

        const amount = new BN(10 * anchor.web3.LAMPORTS_PER_SOL);
        const res1 = await restaking.runUserDepositSOL(user1, amount, null);

        expect(res1.fragSOLFundReserveAccountBalance.sub(res0.fragSOLFundReserveAccountBalance).toNumber()).eq(amount.toNumber(), 'SOL is transferred to fund reserve account');
        expect(res1.fragSOLFund.solOperationReservedAmount.sub(res0.fragSOLFund.solOperationReservedAmount).toString()).eq(amount.toString(), 'fund records correct amount of deposited SOL');

        expect(res1.fragSOLUserFund.user.toString()).eq(user1.publicKey.toString(), 'user check');
        expect(res1.fragSOLUserFund.receiptTokenAmount.toString()).eq(res1.fragSOLUserTokenAccount.amount.toString(), 'user fund records correct amount of minted fragSOL');

        expect(res1.event.userDepositedSolToFund.walletProvider).null;
        expect(res1.event.userDepositedSolToFund.contributionAccrualRate).null;
        expect(res1.event.userDepositedSolToFund.userFundAccount.receiptTokenAmount.toString()).eq(res1.fragSOLUserTokenAccount.amount.toString(), 'fragSOL mint amount in event is valid');
        const mintedAmount = res1.event.userDepositedSolToFund.userFundAccount.receiptTokenAmount.sub(userFundAccount0?.receiptTokenAmount ?? new BN(0));

        expect(res1.event.userUpdatedRewardPool.updatedUserRewardAccountAddresses.length).eq(1, 'user reward account is in event');
        const userRewardAccount1 = await restaking.getUserFragSOLRewardAccount(user1.publicKey);
        expect(userRewardAccount1.userRewardPools1[0].tokenAllocatedAmount.totalAmount.sub(userRewardAccount0?.userRewardPools1[0].tokenAllocatedAmount.totalAmount ?? new BN(0)).toString()).eq(mintedAmount.toString(), 'user reward account updated base pool');
        expect(userRewardAccount1.userRewardPools1[1].tokenAllocatedAmount.totalAmount.sub(userRewardAccount0?.userRewardPools1[1].tokenAllocatedAmount.totalAmount ?? new BN(0)).toString()).eq(mintedAmount.toString(), 'user reward account updated bonus pool');

        const res2 = await restaking.runOperatorUpdatePrices();
        // expect(res2.event.operatorUpdatedFundPrice.fundAccount.oneReceiptTokenAsSol.toString()).eq((mintedAmount.div(new BN(res1.fragSOLUserTokenAccount.amount.toString())).mul(new BN(10 ** restaking.fragSOLDecimals))).toString(), '11');
        expect(res2.fragSOLFundReserveAccountBalance.sub(res0.fragSOLFundReserveAccountBalance).toString()).eq(amount.toString(), '11');
    });

    step("user2 deposits SOL with metadata to mint fragSOL", async function () {
        const res0 = await restaking.runOperatorUpdatePrices();
        const userFundAccount0 = await restaking.getUserFragSOLFundAccount(user2.publicKey).catch(v => null);
        const userRewardAccount0 = await restaking.getUserFragSOLRewardAccount(user2.publicKey).catch(v => null);

        const amount1 = new BN(6 * anchor.web3.LAMPORTS_PER_SOL);
        const currentTimestamp = new BN(Math.floor(Date.now() / 1000));
        const depositMetadata1 = restaking.asType<'depositMetadata'>({
            walletProvider: "BACKPACK",
            contributionAccrualRate: 130,
            expiredAt: currentTimestamp,
        });
        const res1 = await restaking.runUserDepositSOL(user2, amount1, depositMetadata1);

        expect(res1.fragSOLFundReserveAccountBalance.sub(res0.fragSOLFundReserveAccountBalance).toString()).eq(amount1.toString(), 'SOL is transferred to fund reserve account');
        expect(res1.event.userDepositedSolToFund.walletProvider).eq(depositMetadata1.walletProvider, 'wallet provider is correct');
        expect(res1.event.userDepositedSolToFund.contributionAccrualRate.toString()).eq(depositMetadata1.contributionAccrualRate.toString(), 'contribution accrual rate is correct');
        expect(res1.event.userDepositedSolToFund.userFundAccount.receiptTokenAmount.toString()).eq(res1.fragSOLUserTokenAccount.amount.toString(), 'fragSOL mint amount in event is valid');
        const mintedAmount1 = res1.event.userDepositedSolToFund.userFundAccount.receiptTokenAmount.sub(userFundAccount0?.receiptTokenAmount ?? new BN(0));

        expect(res1.event.userUpdatedRewardPool.updatedUserRewardAccountAddresses.length).eq(1, 'user reward account is in event');
        const userRewardAccount1 = await restaking.getUserFragSOLRewardAccount(user2.publicKey);
        expect(userRewardAccount1.userRewardPools1[0].tokenAllocatedAmount.totalAmount.sub(userRewardAccount0?.userRewardPools1[0].tokenAllocatedAmount.totalAmount ?? new BN(0)).toString()).eq(mintedAmount1.toString(), 'user reward account updated base pool');
        expect(userRewardAccount1.userRewardPools1[1].tokenAllocatedAmount.totalAmount.sub(userRewardAccount0?.userRewardPools1[1].tokenAllocatedAmount.totalAmount ?? new BN(0)).toString()).eq(mintedAmount1.toString(), 'user reward account updated bonus pool');

        const amount2 = new BN(4 * anchor.web3.LAMPORTS_PER_SOL);
        const depositMetadata2 = restaking.asType<'depositMetadata'>({
            walletProvider: "FRONTPACK",
            contributionAccrualRate: 110,
            expiredAt: currentTimestamp,
        });
        const res2 = await restaking.runUserDepositSOL(user2, amount2, depositMetadata2);
        const mintedAmount2 = res2.event.userDepositedSolToFund.userFundAccount.receiptTokenAmount.sub(res1.event.userDepositedSolToFund.userFundAccount.receiptTokenAmount);

        expect(res2.fragSOLFundReserveAccountBalance.sub(res1.fragSOLFundReserveAccountBalance).toString()).eq(amount2.toString(), '8');
        expect(res2.event.userDepositedSolToFund.walletProvider).eq(depositMetadata2.walletProvider, '9');
        expect(res2.event.userDepositedSolToFund.contributionAccrualRate.toString()).eq(depositMetadata2.contributionAccrualRate.toString(), '10');
        expect(res2.event.userDepositedSolToFund.userFundAccount.receiptTokenAmount.toString()).eq(res2.fragSOLUserTokenAccount.amount.toString(), '11')

        expect(res2.event.userUpdatedRewardPool.updatedUserRewardAccountAddresses.length).eq(1, '12');
        const userRewardAccount2 = await restaking.getUserFragSOLRewardAccount(user2.publicKey);
        expect(userRewardAccount2.userRewardPools1[0].tokenAllocatedAmount.totalAmount.sub(userRewardAccount1.userRewardPools1[0].tokenAllocatedAmount.totalAmount).toString()).eq(mintedAmount2.toString(), '13');
        expect(userRewardAccount2.userRewardPools1[0].tokenAllocatedAmount.numRecords).eq(1, '14'); // base pool has no custom accrual rate
        expect(userRewardAccount2.userRewardPools1[1].tokenAllocatedAmount.totalAmount.sub(userRewardAccount1.userRewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq(mintedAmount2.toString(), '15');
        expect(userRewardAccount2.userRewardPools1[1].tokenAllocatedAmount.numRecords).eq(2, '16');
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
        const expirationTimestamp = new BN(Math.floor(Date.now() / 1000) - 5); // expired 2 sec ago
        const depositMetadata1 = restaking.asType<'depositMetadata'>({
            walletProvider: "BACKPACK",
            contributionAccrualRate: 130,
            expiredAt: expirationTimestamp,
        });
        await expect(restaking.runUserDepositSOL(user2, amount1, depositMetadata1)).rejectedWith('FundDepositMetadataSignatureExpiredError');
    });
});
