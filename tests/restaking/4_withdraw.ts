import * as anchor from "@coral-xyz/anchor";
import {BN} from '@coral-xyz/anchor';
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";

describe("withdraw", async () => {
    const playground = await restakingPlayground;
    const user5 = playground.keychain.getKeypair('MOCK_USER5');
    const user6 = playground.keychain.getKeypair('MOCK_USER6');

    step("try airdrop SOL to mock accounts", async function () {
        await Promise.all([
            playground.tryAirdrop(user5.publicKey, 100),
            playground.tryAirdrop(user6.publicKey, 100),
        ]);

        await playground.sleep(1); // ...block hash not found?
    });

    const amountSOLDeposited = new BN((10 ** 9) * 20);
    const amountFragSOLWithdrawalEach = new BN((10 ** 9) * 4);
    const withdrawalRequestedSize = 4;

    step("user5 deposits and withdraws", async function () {
        const res0 = await playground.runOperatorUpdatePrices();

        expect(res0.fragSOLFund.withdrawalStatus.numWithdrawalRequestsInProgress.toNumber()).eq(0);
        expect(res0.fragSOLFund.withdrawalStatus.pendingBatchWithdrawal.numWithdrawalRequests.toNumber()).eq(0);
        expect(res0.fragSOLFund.withdrawalStatus.pendingBatchWithdrawal.receiptTokenToProcess.toNumber()).eq(0);

        const res1 = await playground.runUserDepositSOL(user5, amountSOLDeposited, null);
        const account1 = await playground.getUserFragSOLAccount(user5.publicKey);
        expect(res1.event.userDepositedSolToFund.mintedReceiptTokenAmount.toString()).eq(account1.amount.toString());

        const amountFragSOLWithdrawalTotal = amountFragSOLWithdrawalEach.mul(new BN(withdrawalRequestedSize));
        const res2s = await Promise.all(
            Array(withdrawalRequestedSize).fill(null)
                .map((_, i) => playground.sleep(i).then(() => playground.runUserRequestWithdrawal(user5, amountFragSOLWithdrawalEach))),
        );
        const amountWithdrawalActual = res2s.reduce((sum, v) => sum.add(v.event.userRequestedWithdrawalFromFund.requestedReceiptTokenAmount), new BN(0));
        expect(amountWithdrawalActual.toString(), 'withdrawal actual total').eq(amountFragSOLWithdrawalTotal.toString());

        const account2 = await playground.getUserFragSOLAccount(user5.publicKey);
        expect(account2.amount.toString(), 'after balance').eq(new BN(account1.amount.toString()).sub(amountFragSOLWithdrawalTotal).toString(), 'before balance minus total withdrawal amount');

        const res2 = await playground.runOperatorUpdatePrices();
        expect(res2.fragSOLFund.withdrawalStatus.pendingBatchWithdrawal.numWithdrawalRequests.toNumber()).eq(withdrawalRequestedSize);
        expect(res2.fragSOLFund.withdrawalStatus.pendingBatchWithdrawal.solReserved.toNumber()).eq(0, 'not yet processed');
        expect(res2.fragSOLFund.withdrawalStatus.pendingBatchWithdrawal.receiptTokenToProcess.toString()).eq(amountFragSOLWithdrawalTotal.toString());
        expect(res2.fragSOLFund.withdrawalStatus.pendingBatchWithdrawal.receiptTokenBeingProcessed.toNumber()).eq(0);
        expect(res2.fragSOLFund.withdrawalStatus.pendingBatchWithdrawal.receiptTokenProcessed.toNumber()).eq(0);
        expect(res0.fragSOLFund.withdrawalStatus.pendingBatchWithdrawal.batchId.toNumber()).eq(res2.fragSOLFund.withdrawalStatus.pendingBatchWithdrawal.batchId.toNumber());

        const fragSOLLock = await playground.getFragSOLLockAccount();
        expect(res2.fragSOLFund.withdrawalStatus.pendingBatchWithdrawal.receiptTokenToProcess.toString()).eq(fragSOLLock.amount.toString());
        expect(res2.fragSOLFund.withdrawalStatus.pendingBatchWithdrawal.receiptTokenToProcess.sub(res1.fragSOLFund.withdrawalStatus.pendingBatchWithdrawal.receiptTokenToProcess).toString()).eq(amountFragSOLWithdrawalTotal.toString());
    });

    step("user5 cancels withdrawal request", async () => {
        const res0 = await playground.runOperatorUpdatePrices();
        expect(res0.fragSOLFund.withdrawalStatus.pendingBatchWithdrawal.numWithdrawalRequests.toNumber()).eq(withdrawalRequestedSize);

        await expect(playground.runUserCancelWithdrawalRequest(user5, new BN(10))).rejectedWith("FundWithdrawalRequestNotFoundError");

        const res1 = await playground.runUserCancelWithdrawalRequest(user5, new BN(1));
        expect(res1.fragSOLUserFund.withdrawalRequests.length).eq(withdrawalRequestedSize - 1);

        const res2 = await playground.runUserCancelWithdrawalRequest(user5, new BN(3));
        expect(res2.fragSOLUserFund.withdrawalRequests.length).eq(withdrawalRequestedSize - 2);
        expect(res2.fragSOLFund.withdrawalStatus.pendingBatchWithdrawal.numWithdrawalRequests.toNumber()).eq(withdrawalRequestedSize - 2);

        expect(res2.fragSOLFund.withdrawalStatus.pendingBatchWithdrawal.receiptTokenToProcess.toString()).eq(res2.fragSOLLockAccount.amount.toString());
        expect(res2.fragSOLUserFund.receiptTokenAmount.toString()).eq(amountSOLDeposited.sub(amountFragSOLWithdrawalEach.mul(new BN(2))).toString());

        const account2 = await playground.getUserFragSOLAccount(user5.publicKey);
        expect(account2.amount.toString()).eq(res2.fragSOLUserFund.receiptTokenAmount.toString());

        await expect(playground.runUserCancelWithdrawalRequest(user6, new BN(2))).rejectedWith("FundWithdrawalRequestNotFoundError");
    });

    step("user5 (operator) processes queued withdrawals", async () => {
        const res1 = await playground.runOperatorProcessFundWithdrawalJob(user5);

        expect(res1.fragSOLLockAccount.amount.toString()).eq('0');
        expect(res1.fragSOLFund.withdrawalStatus.pendingBatchWithdrawal.numWithdrawalRequests.toNumber()).eq(0);
        expect(res1.fragSOLFund.withdrawalStatus.lastCompletedBatchId.toNumber()).eq(1);
        expect(res1.fragSOLFund.withdrawalStatus.pendingBatchWithdrawal.batchId.toNumber()).eq(2);

        await playground.sleep(1);
        await expect(playground.runOperatorProcessFundWithdrawalJob(user5)).rejectedWith('OperatorJobUnmetThresholdError');

        await playground.sleep(1);
        await expect(playground.runOperatorProcessFundWithdrawalJob(user5, true)).rejectedWith('OperatorJobUnmetThresholdError');

        await playground.sleep(1);
        const res2 = await playground.runOperatorProcessFundWithdrawalJob(playground.keychain.getKeypair('ADMIN'), true);

        expect(res2.fragSOLFund.withdrawalStatus.lastCompletedBatchId.toNumber()).eq(2);
        expect(res2.fragSOLFund.withdrawalStatus.reservedFund.numCompletedWithdrawalRequests.toNumber()).eq(2);
        expect(res2.fragSOLFund.withdrawalStatus.reservedFund.totalReceiptTokenProcessed.toString()).eq(amountFragSOLWithdrawalEach.mul(new BN(2)).toString(), 'in this test, fragSOL unit price is still 1SOL');
        expect(res2.fragSOLFund.withdrawalStatus.reservedFund.solRemaining.toString()).eq(amountFragSOLWithdrawalEach.mul(new BN(2)).toString(), 'in this test, fragSOL unit price is still 1SOL');
        expect(res2.fragSOLLockAccount.amount.toString()).eq('0');
    });

    step("user5 can withdraw SOL", async () => {
        const balance0 = await playground.connection.getBalance(user5.publicKey);
        const res1 = await playground.runUserWithdraw(user5, new BN(2));
        const balance1 = await playground.connection.getBalance(user5.publicKey);
        expect(res1.event.userWithdrewSolFromFund.burntReceiptTokenAmount.toString()).eq(amountFragSOLWithdrawalEach.toString());
        expect(res1.event.userWithdrewSolFromFund.withdrawnSolAmount.toString()).eq((balance1 - balance0).toString());
        // x * (1 - feeRate/10_000) = withdrawnSolAmount
        // x * feeRate/10_000 = deductedSolFeeAmount
        // withdrawnSolAmount/deductedSolFeeAmount = 10_000/feeRate - 1
        expect(res1.event.userWithdrewSolFromFund.withdrawnSolAmount.div(res1.event.userWithdrewSolFromFund.deductedSolFeeAmount).toString())
            .eq((10_000 / res1.fragSOLFund.withdrawalStatus.solWithdrawalFeeRate - 1).toString())
        expect(res1.event.userWithdrewSolFromFund.withdrawnSolAmount.add(res1.event.userWithdrewSolFromFund.deductedSolFeeAmount).toString())
            .eq(amountFragSOLWithdrawalEach.toString(), 'in this test, fragSOL unit price is still 1SOL');
        expect(res1.fragSOLFund.withdrawalStatus.reservedFund.solRemaining.toString())
            .eq(amountFragSOLWithdrawalEach.add(res1.event.userWithdrewSolFromFund.deductedSolFeeAmount).toString(), 'in this test, fragSOL unit price is still 1SOL')
    });

    step("user5 cannot withdraw when withdrawal is disabled", async () => {
        await playground.run({
            instructions: [
                playground.methods
                    .fundManagerUpdateWithdrawalEnabledFlag(false)
                    .instruction(),
            ],
            signerNames: ['FUND_MANAGER'],
        });

        await expect(playground.runUserWithdraw(user5, new BN(4))).rejectedWith('FundWithdrawalDisabledError');

        await playground.run({
            instructions: [
                playground.methods
                    .fundManagerUpdateWithdrawalEnabledFlag(true)
                    .instruction(),
            ],
            signerNames: ['FUND_MANAGER'],
        });

        const res2 = await playground.runUserWithdraw(user5, new BN(4));
        expect(res2.fragSOLFund.withdrawalStatus.pendingBatchWithdrawal.numWithdrawalRequests.toNumber()).eq(0);
        expect(res2.fragSOLFund.withdrawalStatus.reservedFund.solRemaining.toString())
            .eq(amountFragSOLWithdrawalEach.mul(new BN(2 * res2.fragSOLFund.withdrawalStatus.solWithdrawalFeeRate)).div(new BN(10_000)).toString());
    });
});
