import {BN, web3} from '@coral-xyz/anchor';
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";

describe("withdraw SOL", async () => {
    const restaking = await restakingPlayground;
    const user5 = restaking.keychain.getKeypair('MOCK_USER5');
    const user6 = restaking.keychain.getKeypair('MOCK_USER6');

    step("try airdrop SOL to mock accounts", async function () {
        await Promise.all([
            restaking.tryAirdrop(user5.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(100)),
            restaking.tryAirdrop(user6.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(100)),
        ]);

        await restaking.sleep(1); // ...block hash not found?
    });

    const amountSOLDeposited = new BN((10 ** 9) * 20);
    const amountFragSOLWithdrawalEach = new BN((10 ** 9) * 4);
    const withdrawalRequestedSize = 4;

    step("user5 deposits and withdraws", async function () {
        const fragSOLFund0 = await restaking.getFragSOLFundAccount();

        expect(fragSOLFund0.sol.withdrawalPendingBatch.numRequests.toNumber()).eq(0);
        expect(fragSOLFund0.sol.withdrawalPendingBatch.receiptTokenAmount.toNumber()).eq(0);

        const res1 = await restaking.runUserDepositSOL(user5, amountSOLDeposited, null);
        const account1 = await restaking.getUserFragSOLAccount(user5.publicKey);
        expect(res1.event.userDepositedToFund.mintedReceiptTokenAmount.toString()).eq(account1.amount.toString());

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
        expect(fragSOLFund2.sol.withdrawalPendingBatch.numRequests.toNumber()).eq(withdrawalRequestedSize);
        expect(fragSOLFund2.sol.withdrawalUserReservedAmount.toNumber()).eq(0, 'not yet processed');
        expect(fragSOLFund2.sol.withdrawalPendingBatch.receiptTokenAmount.toString()).eq(amountFragSOLWithdrawalTotal.toString());
        expect(fragSOLFund0.sol.withdrawalPendingBatch.batchId.toNumber()).not.eq(fragSOLFund2.sol.withdrawalLastProcessedBatchId.toNumber(), 'not yet processed2');

        const fragSOLLock = await restaking.getFragSOLFundReceiptTokenLockAccount();
        expect(fragSOLFund2.sol.withdrawalPendingBatch.receiptTokenAmount.toString()).eq(fragSOLLock.amount.toString());
        expect(fragSOLFund2.sol.withdrawalPendingBatch.receiptTokenAmount.sub(res1.fragSOLFund.sol.withdrawalPendingBatch.receiptTokenAmount).toString()).eq(amountFragSOLWithdrawalTotal.toString());
    });

    step("user5 cancels withdrawal request", async () => {
        const fragSOLFund0 = await restaking.getFragSOLFundAccount();
        expect(fragSOLFund0.sol.withdrawalPendingBatch.numRequests.toNumber()).eq(withdrawalRequestedSize);

        await expect(restaking.runUserCancelWithdrawalRequest(user5, new BN(10))).rejectedWith("FundWithdrawalRequestNotFoundError");

        const res1 = await restaking.runUserCancelWithdrawalRequest(user5, new BN(1));
        expect(res1.fragSOLUserFund.withdrawalRequests.length).eq(withdrawalRequestedSize - 1);

        const res2 = await restaking.runUserCancelWithdrawalRequest(user5, new BN(3));
        expect(res2.fragSOLUserFund.withdrawalRequests.length).eq(withdrawalRequestedSize - 2);
        expect(res2.fragSOLFund.sol.withdrawalPendingBatch.numRequests.toNumber()).eq(withdrawalRequestedSize - 2);

        expect(res2.fragSOLFund.sol.withdrawalPendingBatch.receiptTokenAmount.toString()).eq(res2.fragSOLLockAccount.amount.toString());
        expect(res2.fragSOLUserFund.receiptTokenAmount.toString()).eq(amountSOLDeposited.sub(amountFragSOLWithdrawalEach.mul(new BN(2))).toString());

        const account2 = await restaking.getUserFragSOLAccount(user5.publicKey);
        expect(account2.amount.toString()).eq(res2.fragSOLUserFund.receiptTokenAmount.toString());

        await expect(restaking.runUserCancelWithdrawalRequest(user6, new BN(2))).rejectedWith("FundWithdrawalRequestNotFoundError");
    });

    step("user5 (operator) processes queued withdrawals", async () => {
        const programRevenueAmount0 = await restaking.getProgramRevenueAccountBalance().catch(_ => new BN(0));
        const res1 = await restaking.runOperatorProcessWithdrawalBatches();

        expect(res1.fragSOLLockAccount.amount.toString()).eq('0');
        expect(res1.fragSOLFund.sol.withdrawalPendingBatch.numRequests.toNumber()).eq(0);

        await restaking.sleep(1);
        const res2 = await restaking.runOperatorProcessWithdrawalBatches();
        expect(res2.fragSOLFund.sol.withdrawalLastProcessedBatchId.toNumber()).eq(res1.fragSOLFund.sol.withdrawalLastProcessedBatchId.toNumber());

        await restaking.sleep(1);
        await expect(restaking.runOperatorProcessWithdrawalBatches(user5, true)).rejectedWith('FundOperationUnauthorizedCommandError');

        await restaking.sleep(1);
        const res3 = await restaking.runOperatorProcessWithdrawalBatches(restaking.keychain.getKeypair('FUND_MANAGER'), true);
        const programRevenueAmount1 = await restaking.getProgramRevenueAccountBalance();

        expect(res3.fragSOLFund.sol.withdrawalLastProcessedBatchId.toNumber()).eq(res1.fragSOLFund.sol.withdrawalPendingBatch.batchId.toNumber() - 1, 'no processing with no requests');
        expect(res3.fragSOLFund.sol.withdrawalUserReservedAmount.toString()).eq(amountFragSOLWithdrawalEach.muln(2).muln(10000 - res3.fragSOLFund.withdrawalFeeRateBps).divn(10000).toString(), 'in this test, fragSOL unit price is still 1SOL');
        expect(res3.fragSOLLockAccount.amount.toString()).eq('0');
        expect(
            amountFragSOLWithdrawalEach.mul(new BN(2)).muln(res3.fragSOLFund.withdrawalFeeRateBps).divn(10_000)
                .sub(programRevenueAmount1.sub(programRevenueAmount0)).toNumber()
        ).lt(10, 'check fee; here 1SOL=1RT');
    });

    step("user5 can withdraw SOL", async () => {
        const balance0 = await restaking.connection.getBalance(user5.publicKey);
        const res1 = await restaking.runUserWithdraw(user5, new BN(2));
        const balance1 = await restaking.connection.getBalance(user5.publicKey);
        expect(res1.event.userWithdrewFromFund.burntReceiptTokenAmount.toString()).eq(amountFragSOLWithdrawalEach.toString());
        expect(res1.event.userWithdrewFromFund.withdrawnAmount.toString(), 'event').eq((balance1 - balance0).toString(), 'balance diff');
        // x * (1 - feeRate/10_000) = withdrawnSolAmount
        // x * feeRate/10_000 = deductedSolFeeAmount
        // withdrawnSolAmount/deductedSolFeeAmount = 10_000/feeRate - 1
        expect(res1.event.userWithdrewFromFund.withdrawnAmount.div(res1.event.userWithdrewFromFund.deductedFeeAmount).toString())
            .eq((10_000 / res1.fragSOLFund.withdrawalFeeRateBps - 1).toString(), '2');
        expect(res1.event.userWithdrewFromFund.withdrawnAmount.add(res1.event.userWithdrewFromFund.deductedFeeAmount).toString())
            .eq(amountFragSOLWithdrawalEach.toString(), 'in this test, fragSOL unit price is still 1SOL - 1');
        expect(res1.event.userWithdrewFromFund.withdrawnAmount.toString())
            .eq(amountFragSOLWithdrawalEach.sub(res1.event.userWithdrewFromFund.deductedFeeAmount).toString(), '3');
        expect(res1.fragSOLFund.sol.withdrawalUserReservedAmount.toString())
            .eq(amountFragSOLWithdrawalEach.muln(10000 - res1.fragSOLFund.withdrawalFeeRateBps).divn(10000).toString(), 'in this test, fragSOL unit price is still 1SOL - 2');
        expect(amountFragSOLWithdrawalEach.toString())
            .eq(res1.event.userWithdrewFromFund.withdrawnAmount.add(res1.event.userWithdrewFromFund.deductedFeeAmount).toString(), '4');
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
        expect(res2.fragSOLFund.sol.withdrawalPendingBatch.numRequests.toNumber()).eq(0);
    });
});
