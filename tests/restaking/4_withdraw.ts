import {BN} from '@coral-xyz/anchor';
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";

describe("withdraw", async () => {
    const restaking = await restakingPlayground;
    const user5 = restaking.keychain.getKeypair('MOCK_USER5');
    const user6 = restaking.keychain.getKeypair('MOCK_USER6');

    step("try airdrop SOL to mock accounts", async function () {
        await Promise.all([
            restaking.tryAirdrop(user5.publicKey, 100),
            restaking.tryAirdrop(user6.publicKey, 100),
        ]);

        await restaking.sleep(1); // ...block hash not found?
    });

    const amountSOLDeposited = new BN((10 ** 9) * 20);
    const amountFragSOLWithdrawalEach = new BN((10 ** 9) * 4);
    const withdrawalRequestedSize = 4;

    step("user5 deposits and withdraws", async function () {
        const fragSOLFund0 = await restaking.getFragSOLFundAccount();

        expect(fragSOLFund0.withdrawal.numRequestsInProgress.toNumber()).eq(0);
        expect(fragSOLFund0.withdrawal.pendingBatch.numRequests.toNumber()).eq(0);
        expect(fragSOLFund0.withdrawal.pendingBatch.receiptTokenAmount.toNumber()).eq(0);

        const res1 = await restaking.runUserDepositSOL(user5, amountSOLDeposited, null);
        const account1 = await restaking.getUserFragSOLAccount(user5.publicKey);
        expect(res1.event.userDepositedSolToFund.mintedReceiptTokenAmount.toString()).eq(account1.amount.toString());

        const amountFragSOLWithdrawalTotal = amountFragSOLWithdrawalEach.mul(new BN(withdrawalRequestedSize));
        const res2s = await Promise.all(
            Array(withdrawalRequestedSize).fill(null)
                .map((_, i) => restaking.sleep(i).then(() => restaking.runUserRequestWithdrawal(user5, amountFragSOLWithdrawalEach))),
        );
        const amountWithdrawalActual = res2s.reduce((sum, v) => sum.add(v.event.userRequestedWithdrawalFromFund.requestedReceiptTokenAmount), new BN(0));
        expect(amountWithdrawalActual.toString(), 'withdrawal actual total').eq(amountFragSOLWithdrawalTotal.toString());

        const account2 = await restaking.getUserFragSOLAccount(user5.publicKey);
        expect(account2.amount.toString(), 'after balance').eq(new BN(account1.amount.toString()).sub(amountFragSOLWithdrawalTotal).toString(), 'before balance minus total withdrawal amount');

        const fragSOLFund2 = await restaking.getFragSOLFundAccount();
        expect(fragSOLFund2.withdrawal.pendingBatch.numRequests.toNumber()).eq(withdrawalRequestedSize);
        expect(fragSOLFund2.withdrawal.solWithdrawalReservedAmount.toNumber()).eq(0, 'not yet processed');
        expect(fragSOLFund2.withdrawal.pendingBatch.receiptTokenAmount.toString()).eq(amountFragSOLWithdrawalTotal.toString());
        expect(fragSOLFund0.withdrawal.pendingBatch.batchId.toNumber()).eq(fragSOLFund2.withdrawal.pendingBatch.batchId.toNumber());

        const fragSOLLock = await restaking.getFragSOLFundReceiptTokenLockAccount();
        expect(fragSOLFund2.withdrawal.pendingBatch.receiptTokenAmount.toString()).eq(fragSOLLock.amount.toString());
        expect(fragSOLFund2.withdrawal.pendingBatch.receiptTokenAmount.sub(res1.fragSOLFund.withdrawal.pendingBatch.receiptTokenAmount).toString()).eq(amountFragSOLWithdrawalTotal.toString());
    });

    step("user5 cancels withdrawal request", async () => {
        const fragSOLFund0 = await restaking.getFragSOLFundAccount();
        expect(fragSOLFund0.withdrawal.pendingBatch.numRequests.toNumber()).eq(withdrawalRequestedSize);

        await expect(restaking.runUserCancelWithdrawalRequest(user5, new BN(10))).rejectedWith("FundWithdrawalRequestNotFoundError");

        const res1 = await restaking.runUserCancelWithdrawalRequest(user5, new BN(1));
        expect(res1.fragSOLUserFund.withdrawalRequests.length).eq(withdrawalRequestedSize - 1);

        const res2 = await restaking.runUserCancelWithdrawalRequest(user5, new BN(3));
        expect(res2.fragSOLUserFund.withdrawalRequests.length).eq(withdrawalRequestedSize - 2);
        expect(res2.fragSOLFund.withdrawal.pendingBatch.numRequests.toNumber()).eq(withdrawalRequestedSize - 2);

        expect(res2.fragSOLFund.withdrawal.pendingBatch.receiptTokenAmount.toString()).eq(res2.fragSOLLockAccount.amount.toString());
        expect(res2.fragSOLUserFund.receiptTokenAmount.toString()).eq(amountSOLDeposited.sub(amountFragSOLWithdrawalEach.mul(new BN(2))).toString());

        const account2 = await restaking.getUserFragSOLAccount(user5.publicKey);
        expect(account2.amount.toString()).eq(res2.fragSOLUserFund.receiptTokenAmount.toString());

        await expect(restaking.runUserCancelWithdrawalRequest(user6, new BN(2))).rejectedWith("FundWithdrawalRequestNotFoundError");
    });

    step("user5 (operator) processes queued withdrawals", async () => {
        const res1 = await restaking.runOperatorProcessFundWithdrawalJob();

        expect(res1.fragSOLLockAccount.amount.toString()).eq('0');
        expect(res1.fragSOLFund.withdrawal.pendingBatch.numRequests.toNumber()).eq(0);
        expect(res1.fragSOLFund.withdrawal.lastProcessedBatchId.toNumber()).eq(1);
        expect(res1.fragSOLFund.withdrawal.pendingBatch.batchId.toNumber()).eq(2);

        await restaking.sleep(1);
        await expect(restaking.runOperatorProcessFundWithdrawalJob()).rejectedWith('OperatorJobUnmetThresholdError');

        await restaking.sleep(1);
        await expect(restaking.runOperatorProcessFundWithdrawalJob(user5, true)).rejectedWith('RequireEqViolated');

        await restaking.sleep(1);
        const res2 = await restaking.runOperatorProcessFundWithdrawalJob(restaking.keychain.getKeypair('FUND_MANAGER'), true);

        expect(res2.fragSOLFund.withdrawal.lastProcessedBatchId.toNumber()).eq(2);
        expect(res2.fragSOLFund.withdrawal.solWithdrawalReservedAmount.toString()).eq(amountFragSOLWithdrawalEach.mul(new BN(2)).toString(), 'in this test, fragSOL unit price is still 1SOL');
        expect(res2.fragSOLLockAccount.amount.toString()).eq('0');
    });

    step("user5 can withdraw SOL", async () => {
        const balance0 = await restaking.connection.getBalance(user5.publicKey);
        const res1 = await restaking.runUserWithdraw(user5, new BN(2));
        const balance1 = await restaking.connection.getBalance(user5.publicKey);
        expect(res1.event.userWithdrewSolFromFund.burntReceiptTokenAmount.toString()).eq(amountFragSOLWithdrawalEach.toString());
        expect(res1.event.userWithdrewSolFromFund.withdrawnSolAmount.toString(), 'event').eq((balance1 - balance0).toString(), 'balance diff');
        // x * (1 - feeRate/10_000) = withdrawnSolAmount
        // x * feeRate/10_000 = deductedSolFeeAmount
        // withdrawnSolAmount/deductedSolFeeAmount = 10_000/feeRate - 1
        expect(res1.event.userWithdrewSolFromFund.withdrawnSolAmount.div(res1.event.userWithdrewSolFromFund.deductedSolFeeAmount).toString())
            .eq((10_000 / res1.fragSOLFund.withdrawal.solFeeRateBps - 1).toString(), '2');
        expect(res1.event.userWithdrewSolFromFund.withdrawnSolAmount.add(res1.event.userWithdrewSolFromFund.deductedSolFeeAmount).toString())
            .eq(amountFragSOLWithdrawalEach.toString(), 'in this test, fragSOL unit price is still 1SOL - 1');
        expect(res1.event.userWithdrewSolFromFund.withdrawnSolAmount.toString())
            .eq(amountFragSOLWithdrawalEach.sub(res1.event.userWithdrewSolFromFund.deductedSolFeeAmount).toString(), '3');
        expect(res1.fragSOLFund.withdrawal.solWithdrawalReservedAmount.toString())
            .eq(amountFragSOLWithdrawalEach.toString(), 'in this test, fragSOL unit price is still 1SOL - 2');
        expect(res1.fragSOLFund.withdrawal.solWithdrawalReservedAmount.toString())
            .eq(res1.event.userWithdrewSolFromFund.withdrawnSolAmount.add(res1.event.userWithdrewSolFromFund.deductedSolFeeAmount).toString(), '4');
    });

    step("user5 cannot request withdrawal when withdrawal is disabled", async () => {
        const fragSOLFundAccount = await restaking.getFragSOLFundAccount();
        await restaking.run({
            instructions: [
                restaking.methods
                    .fundManagerUpdateFundStrategy(
                        fragSOLFundAccount.solAccumulatedDepositCapacityAmount,
                        fragSOLFundAccount.withdrawal.solFeeRateBps,
                        0,
                        new BN(0),
                        fragSOLFundAccount.withdrawal.batchThresholdIntervalSeconds,
                        false,
                    )
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
                        fragSOLFundAccount.solAccumulatedDepositCapacityAmount,
                        fragSOLFundAccount.withdrawal.solFeeRateBps,
                        0,
                        new BN(0),
                        fragSOLFundAccount.withdrawal.batchThresholdIntervalSeconds,
                        true,
                    )
                    .instruction(),
            ],
            signerNames: ['FUND_MANAGER'],
            events: ['fundManagerUpdatedFund'],
        });

        const res2 = await restaking.runUserWithdraw(user5, new BN(4));
        expect(res2.fragSOLFund.withdrawal.pendingBatch.numRequests.toNumber()).eq(0);
    });
});
