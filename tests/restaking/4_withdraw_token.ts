import {BN, web3} from '@coral-xyz/anchor';
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";

describe("withdraw token", async () => {
    const restaking = await restakingPlayground;
    const user5 = restaking.keychain.getKeypair('MOCK_USER5');
    const user6= restaking.keychain.getKeypair('MOCK_USER6');

    step("try airdrop SOL and tokens to mock accounts", async function () {
        await Promise.all([
            restaking.tryAirdrop(user5.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(100)),
            restaking.tryAirdropSupportedTokens(user5.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(100)),
            restaking.tryAirdrop(user6.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(100)),
        ]);

        await restaking.sleep(1); // ...block hash not found?
    });

    const amountTokenDeposited = new BN((10 ** 9) * 20);
    const amountFragSOLWithdrawalEach = new BN((10 ** 9) * 4);
    const withdrawalRequestedSize = 4;

    step("user5 deposits token", async function () {
        const fragSOLFund0 = await restaking.getFragSOLFundAccount();

        expect(fragSOLFund0.supportedTokens[0].token.withdrawalPendingBatch.numRequests.toNumber()).eq(0);
        expect(fragSOLFund0.supportedTokens[0].token.withdrawalPendingBatch.receiptTokenAmount.toNumber()).eq(0);

        const res1 = await restaking.runUserDepositSupportedToken(user5, 'bSOL', amountTokenDeposited, null);
        const account1 = await restaking.getUserFragSOLAccount(user5.publicKey);
        expect(res1.event.userDepositedToFund.mintedReceiptTokenAmount.toString()).eq(account1.amount.toString());
        expect(res1.fragSOLFund.supportedTokens[0].token.withdrawableValueAsReceiptTokenAmount.toString()).eq(amountTokenDeposited.toString(), 'withdrawable token amount 1');
    });

    step("user5 cannot withdraw non-withdrawable token", async function () {
        const fragSOLFund0 = await restaking.getFragSOLFundAccount();
        let supportedToken = fragSOLFund0.supportedTokens[0];
        expect(restaking.runUserRequestWithdrawal(user5, amountFragSOLWithdrawalEach, supportedToken.mint)).rejectedWith("FundWithdrawalNotSupportedAsset");

        // now turn on withdrawable
        await restaking.run({
            instructions: [
                restaking.methods
                    .fundManagerUpdateSupportedTokenStrategy(
                        supportedToken.mint,
                        true,
                        supportedToken.token.accumulatedDepositCapacityAmount,
                        null, // Option<token_accumulated_deposit_amount>
                        true, // withdrawable,
                        supportedToken.token.normalReserveRateBps,
                        supportedToken.token.normalReserveMaxAmount,
                        supportedToken.rebalancingAmount,
                        supportedToken.solAllocationWeight,
                        supportedToken.solAllocationCapacityAmount,
                    )
                    .accountsPartial({
                        receiptTokenMint: restaking.knownAddress.fragSOLTokenMint,
                    })
                    .instruction(),
            ],
            signerNames: ['FUND_MANAGER'],
            events: ['fundManagerUpdatedFund'],
        });
    });

    step("user5 withdraws token", async function () {
        const fragSOLFund0 = await restaking.getFragSOLFundAccount();
        const account1 = await restaking.getUserFragSOLAccount(user5.publicKey);

        const amountFragSOLWithdrawalTotal = amountFragSOLWithdrawalEach.mul(new BN(withdrawalRequestedSize));
        const res2s = await Promise.all(
            Array(withdrawalRequestedSize).fill(null)
                .map((_, i) => restaking.sleep(i).then(() => restaking.runUserRequestWithdrawal(user5, amountFragSOLWithdrawalEach, fragSOLFund0.supportedTokens[0].mint))),
        );
        const amountWithdrawalActual = res2s.reduce((sum, v) => sum.add(v.event.userRequestedWithdrawalFromFund.requestedReceiptTokenAmount), new BN(0));
        expect(amountWithdrawalActual.toString(), 'withdrawal actual total').eq(amountFragSOLWithdrawalTotal.toString());

        const account2 = await restaking.getUserFragSOLAccount(user5.publicKey);
        expect(account2.amount.toString(), 'after balance').eq(new BN(account1.amount.toString()).sub(amountFragSOLWithdrawalTotal).toString(), 'before balance minus total withdrawal amount');

        const fragSOLFund2 = await restaking.getFragSOLFundAccount();
        expect(fragSOLFund2.supportedTokens[0].token.withdrawalPendingBatch.numRequests.toNumber()).eq(withdrawalRequestedSize);
        expect(fragSOLFund2.supportedTokens[0].token.withdrawalUserReservedAmount.toNumber()).eq(0, 'not yet processed');
        expect(fragSOLFund2.supportedTokens[0].token.withdrawalPendingBatch.receiptTokenAmount.toString()).eq(amountFragSOLWithdrawalTotal.toString());
        expect(fragSOLFund0.supportedTokens[0].token.withdrawalPendingBatch.batchId.toNumber()).not.eq(fragSOLFund2.supportedTokens[0].token.withdrawalLastProcessedBatchId.toNumber(), 'not yet processed2');

        const fragSOLLock = await restaking.getFragSOLFundReceiptTokenLockAccount();
        expect(fragSOLFund2.supportedTokens[0].token.withdrawalPendingBatch.receiptTokenAmount.toString()).eq(fragSOLLock.amount.toString());
    });

    step("user5 cancels token withdrawal request", async () => {
        const fragSOLFund0 = await restaking.getFragSOLFundAccount();
        expect(fragSOLFund0.supportedTokens[0].token.withdrawalPendingBatch.numRequests.toNumber()).eq(withdrawalRequestedSize);

        await expect(restaking.runUserCancelWithdrawalRequest(user5, new BN(10))).rejectedWith("FundWithdrawalRequestNotFoundError");

        const res1 = await restaking.runUserCancelWithdrawalRequest(user5, new BN(1));
        expect(res1.fragSOLUserFund.withdrawalRequests.length).eq(withdrawalRequestedSize - 1, '1');

        const res2 = await restaking.runUserCancelWithdrawalRequest(user5, new BN(3));
        expect(res2.fragSOLUserFund.withdrawalRequests.length).eq(withdrawalRequestedSize - 2, '2');
        expect(res2.fragSOLFund.supportedTokens[0].token.withdrawalPendingBatch.numRequests.toNumber()).eq(withdrawalRequestedSize - 2);

        expect(res2.fragSOLFund.supportedTokens[0].token.withdrawalPendingBatch.receiptTokenAmount.toString()).eq(res2.fragSOLLockAccount.amount.toString(), '3');
        expect(res2.fragSOLUserFund.receiptTokenAmount.divn(10).toString()).eq(amountTokenDeposited.mul(fragSOLFund0.supportedTokens[0].oneTokenAsSol).div(fragSOLFund0.oneReceiptTokenAsSol).sub(amountFragSOLWithdrawalEach.mul(new BN(2))).divn(10).toString(), '4');
        expect(res2.fragSOLFund.supportedTokens[0].token.withdrawableValueAsReceiptTokenAmount.toString()).eq(amountFragSOLWithdrawalEach.muln(3).toString(), 'withdrawable token amount 2');

        const account2 = await restaking.getUserFragSOLAccount(user5.publicKey);
        expect(account2.amount.toString()).eq(res2.fragSOLUserFund.receiptTokenAmount.toString(), '5');

        await expect(restaking.runUserCancelWithdrawalRequest(user6, new BN(2))).rejectedWith("FundWithdrawalRequestNotFoundError");
    });


    step("user5 (operator) processes queued withdrawals", async () => {
        const programRevenueAmount0 = await restaking.getProgramSupportedTokenRevenueAccountBalance('bSOL').catch(_ => new BN(0));
        const fragSOLFund0 = await restaking.getFragSOLFundAccount();
        const res1 = await restaking.runOperatorProcessWithdrawalBatches(fragSOLFund0.supportedTokens[0].mint);

        expect(res1.fragSOLLockAccount.amount.toString()).eq('0');
        expect(res1.fragSOLFund.supportedTokens[0].token.withdrawalPendingBatch.numRequests.toNumber()).eq(0);

        await restaking.sleep(1);
        const res2 = await restaking.runOperatorProcessWithdrawalBatches(fragSOLFund0.supportedTokens[0].mint);
        expect(res2.fragSOLFund.supportedTokens[0].token.withdrawalLastProcessedBatchId.toNumber()).eq(res1.fragSOLFund.supportedTokens[0].token.withdrawalLastProcessedBatchId.toNumber(), '1');

        await restaking.sleep(1);
        await expect(restaking.runOperatorProcessWithdrawalBatches(fragSOLFund0.supportedTokens[0].mint, user5, true)).rejectedWith('FundOperationUnauthorizedCommandError');

        await restaking.sleep(1);
        const res3 = await restaking.runOperatorProcessWithdrawalBatches(fragSOLFund0.supportedTokens[0].mint, restaking.keychain.getKeypair('FUND_MANAGER'), true);

        expect(res3.fragSOLFund.supportedTokens[0].token.withdrawalLastProcessedBatchId.toNumber()).eq(res1.fragSOLFund.supportedTokens[0].token.withdrawalPendingBatch.batchId.toNumber() - 1, 'no processing with no requests');
        // TODO: expect(res3.fragSOLFund.supportedTokens[0].token.withdrawalUserReservedAmount.toString()).eq(amountFragSOLWithdrawalEach.muln(2).muln(10000 - res3.fragSOLFund.withdrawalFeeRateBps).divn(10000).toString(), 'in this test, fragSOL unit price is still 1SOL');
        expect(res3.fragSOLLockAccount.amount.toString()).eq('0');

        const programRevenueAmount1 = await restaking.getProgramSupportedTokenRevenueAccountBalance('bSOL');
        expect(
            amountFragSOLWithdrawalEach.mul(new BN(2)).muln(res3.fragSOLFund.withdrawalFeeRateBps).divn(10_000)
                .sub(programRevenueAmount1.sub(programRevenueAmount0)).toNumber()
        ).lt(10, 'check fee; here 1ST=1RT');
    });

    step("user5 can withdraw token", async () => {

        const fragSOLFund0 = await restaking.getFragSOLFundAccount();
        const balance0 = await restaking.getUserSupportedTokenAccount(user5.publicKey, 'bSOL').then(a => a.amount);
        const res1 = await restaking.runUserWithdraw(user5, new BN(2));
        const balance1 = await restaking.getUserSupportedTokenAccount(user5.publicKey, 'bSOL').then(a => a.amount);
        expect(res1.event.userWithdrewFromFund.burntReceiptTokenAmount.toString()).eq(amountFragSOLWithdrawalEach.toString());
        expect(res1.event.userWithdrewFromFund.withdrawnAmount.toString(), 'event').eq((balance1 - balance0).toString(), 'balance diff');
        // x * (1 - feeRate/10_000) = withdrawnSolAmount
        // x * feeRate/10_000 = deductedSolFeeAmount
        // withdrawnSolAmount/deductedSolFeeAmount = 10_000/feeRate - 1
        expect(res1.event.userWithdrewFromFund.withdrawnAmount.div(res1.event.userWithdrewFromFund.deductedFeeAmount).toString())
            .eq((10_000 / res1.fragSOLFund.withdrawalFeeRateBps - 1).toString(), '2');
        // TODO: expect(res1.event.userWithdrewFromFund.withdrawnAmount.add(res1.event.userWithdrewFromFund.deductedFeeAmount).toString())
        //     .eq(amountFragSOLWithdrawalEach.toString(), 'in this test, fragSOL unit price is still 1SOL - 1');
        expect(res1.event.userWithdrewFromFund.withdrawnAmount.divn(10).toString())
            .eq(amountFragSOLWithdrawalEach.mul(fragSOLFund0.oneReceiptTokenAsSol).div(fragSOLFund0.supportedTokens[0].oneTokenAsSol).sub(res1.event.userWithdrewFromFund.deductedFeeAmount).divn(10).toString(), '3');
        // TODO: expect(res1.fragSOLFund.supportedTokens[0].token.withdrawalUserReservedAmount.toString())
        //     .eq(amountFragSOLWithdrawalEach.muln(10000 - res1.fragSOLFund.withdrawalFeeRateBps).divn(10000).toString(), 'in this test, fragSOL unit price is still 1SOL - 2');

        expect(
            amountFragSOLWithdrawalEach.mul(fragSOLFund0.oneReceiptTokenAsSol).div(fragSOLFund0.supportedTokens[0].oneTokenAsSol)
                .sub(
                    res1.event.userWithdrewFromFund.withdrawnAmount.add(res1.event.userWithdrewFromFund.deductedFeeAmount)
                ).toNumber()
        ).lt(10, '4');
    });

    step("user5 cannot request withdrawal when withdrawal is disabled", async () => {
        const fragSOLFundAccount = await restaking.getFragSOLFundAccount();
        await restaking.run({
            instructions: [
                restaking.methods
                    .fundManagerUpdateFundStrategy(
                        true,
                        false,
                        fragSOLFundAccount.withdrawalFeeRateBps,
                        fragSOLFundAccount.withdrawalBatchThresholdIntervalSeconds,
                    )
                    .accountsPartial({
                        receiptTokenMint: restaking.knownAddress.fragSOLTokenMint,
                    })
                    .instruction(),
            ],
            signerNames: ['FUND_MANAGER'],
            events: ['fundManagerUpdatedFund'],
        });

        await expect(restaking.runUserRequestWithdrawal(user5, amountFragSOLWithdrawalEach)).rejectedWith('FundWithdrawalDisabledError');

        await restaking.run({
            instructions: [
                restaking.methods
                    .fundManagerUpdateFundStrategy(
                        true,
                        true,
                        fragSOLFundAccount.withdrawalFeeRateBps,
                        fragSOLFundAccount.withdrawalBatchThresholdIntervalSeconds,
                    )
                    .accountsPartial({
                        receiptTokenMint: restaking.knownAddress.fragSOLTokenMint,
                    })
                    .instruction(),
            ],
            signerNames: ['FUND_MANAGER'],
            events: ['fundManagerUpdatedFund'],
        });

        const res2 = await restaking.runUserWithdraw(user5, new BN(4));
        expect(res2.fragSOLFund.supportedTokens[0].token.withdrawalPendingBatch.numRequests.toNumber()).eq(0);
    });
});
