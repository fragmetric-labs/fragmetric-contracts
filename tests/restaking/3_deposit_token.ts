import * as anchor from "@coral-xyz/anchor";
import { BN } from '@coral-xyz/anchor';
import { expect } from "chai";
import {RestakingPlayground} from "../../tools/restaking/playground";

describe("deposit_token", async () => {
    const playground = await RestakingPlayground.local(anchor.AnchorProvider.env());
    const user3 = playground.keychain.getKeypair('MOCK_USER3');
    const user4 = playground.keychain.getKeypair('MOCK_USER4');

    it("try airdrop SOL and supported tokens to mock accounts", async function () {
        await Promise.all([
            playground.tryAirdrop(user3.publicKey, 100),
            playground.tryAirdrop(user4.publicKey, 100),
        ]);

        await playground.sleep(1); // ...block hash not found?

        await Promise.all([
            playground.tryAirdropSupportedTokens(user3.publicKey, 100_000),
            playground.tryAirdropSupportedTokens(user4.publicKey, 100_000),
        ]);

        await playground.sleep(1);
    });

    it("user3 deposits supported token without metadata to mint fragSOL", async function () {
        const res0 = await playground.runOperatorUpdatePrices();
        expect(res0.event.operatorUpdatedFundPrice.fundAccount.receiptTokenPrice.toNumber()).greaterThan(0);
        expect(res0.fragSOLFundBalance.toNumber()).greaterThan(0);
        const fragSOLPrice0 = res0.event.operatorUpdatedFundPrice.fundAccount.receiptTokenPrice;

        const decimals = 10 ** 9;
        const amount = new BN(10 * decimals);
        const symbol = 'bSOL';
        const initialTokenAmount = new BN((await playground.getUserSupportedTokenAccount(user3.publicKey, symbol)).amount.toString());
        const res1 = await playground.runUserDepositSupportedToken(user3, symbol, amount, null);

        expect(new BN(res1.userSupportedTokenAccount.amount.toString()).toString()).eq(new BN(initialTokenAmount).sub(amount).toString());
        expect(res1.fragSOLFund.supportedTokens[0].operationReservedAmount.toString()).eq(amount.toString());
        const fragSOLPrice1 = res1.event.userDepositedSupportedTokenToFund.fundAccount.receiptTokenPrice;

        expect(fragSOLPrice0.toString()).eq(fragSOLPrice1.toString()); // price is consistent upon deposits

        const tokenPrice = res1.fragSOLFund.supportedTokens[0].price;
        expect(tokenPrice.mul(amount).div(fragSOLPrice1).toString()).eq(res1.fragSOLUserTokenAccount.amount.toString()); // proper amount minted?

        expect(res1.event.userDepositedSupportedTokenToFund.walletProvider).null;
        expect(res1.event.userDepositedSupportedTokenToFund.contributionAccrualRate).null;
        expect(res1.event.userUpdatedRewardPool.updates.length).eq(1);
        expect(res1.event.userUpdatedRewardPool.updates[0].updatedUserRewardPools.length).eq(2);
    });

    it("user3 fails to deposit too many tokens", async function () {
        const decimals = 10 ** 9;
        const amount = new BN((10 ** 4) * decimals);
        const symbol = 'bSOL';
        await expect(playground.runUserDepositSupportedToken(user3, symbol, amount, null)).rejectedWith('FundExceededTokenCapacityAmountError');
    });

    it("user4 deposits supported token with metadata to mint fragSOL", async function () {
        const res0 = await playground.runOperatorUpdatePrices();

        const symbol = 'bSOL';
        const decimals = 10 ** 9;
        const amount1 = new BN(6 * decimals);
        const depositMetadata1 = playground.asType<'depositMetadata'>({
            walletProvider: "BACKPACK",
            contributionAccrualRate: 130,
        });
        const res1 = await playground.runUserDepositSupportedToken(user4, symbol, amount1, depositMetadata1);
        const mintedAmount1 = res1.event.userDepositedSupportedTokenToFund.mintedReceiptTokenAmount;

        expect(res1.fragSOLFund.supportedTokens[0].operationReservedAmount.sub(res0.fragSOLFund.supportedTokens[0].operationReservedAmount).toString()).eq(amount1.toString());
        expect(res1.event.userDepositedSupportedTokenToFund.walletProvider).eq(depositMetadata1.walletProvider);
        expect(res1.event.userDepositedSupportedTokenToFund.contributionAccrualRate.toString()).eq(depositMetadata1.contributionAccrualRate.toString());
        expect(res1.event.userDepositedSupportedTokenToFund.userFundAccount.receiptTokenAmount.toString()).eq(res1.fragSOLUserTokenAccount.amount.toString())

        expect(res1.event.userUpdatedRewardPool.updates.length).eq(1);
        expect(res1.event.userUpdatedRewardPool.updates[0].updatedUserRewardPools.length).eq(2);
        expect(res1.event.userUpdatedRewardPool.updates[0].updatedUserRewardPools[0].tokenAllocatedAmount.totalAmount.toString()).eq(mintedAmount1.toString());
        expect(res1.event.userUpdatedRewardPool.updates[0].updatedUserRewardPools[1].tokenAllocatedAmount.totalAmount.toString()).eq(mintedAmount1.toString());

        const amount2 = new BN(4 * decimals);
        const depositMetadata2 = playground.asType<'depositMetadata'>({
            walletProvider: "FRONTPACK",
            contributionAccrualRate: 110,
        });
        const res2 = await playground.runUserDepositSupportedToken(user4, symbol, amount2, depositMetadata2);
        const mintedAmount2 = res2.event.userDepositedSupportedTokenToFund.mintedReceiptTokenAmount;

        expect(res2.fragSOLFund.supportedTokens[0].operationReservedAmount.sub(res1.fragSOLFund.supportedTokens[0].operationReservedAmount).toString(), 'added reserved token amount').eq(amount2.toString(), 'deposited token amount');
        expect(res2.event.userDepositedSupportedTokenToFund.walletProvider).eq(depositMetadata2.walletProvider);
        expect(res2.event.userDepositedSupportedTokenToFund.contributionAccrualRate.toString()).eq(depositMetadata2.contributionAccrualRate.toString());
        expect(res2.event.userDepositedSupportedTokenToFund.userFundAccount.receiptTokenAmount.toString()).eq(res2.fragSOLUserTokenAccount.amount.toString())

        expect(res2.event.userUpdatedRewardPool.updates.length).eq(1);
        expect(res2.event.userUpdatedRewardPool.updates[0].updatedUserRewardPools.length).eq(2);
        expect(res2.event.userUpdatedRewardPool.updates[0].updatedUserRewardPools[0].tokenAllocatedAmount.totalAmount.toString(), 'total allocated amount').eq(mintedAmount1.add(mintedAmount2).toString(), 'minted fragSOL amount');
        expect(res2.event.userUpdatedRewardPool.updates[0].updatedUserRewardPools[0].tokenAllocatedAmount.numRecords).eq(1);
        expect(res2.event.userUpdatedRewardPool.updates[0].updatedUserRewardPools[1].tokenAllocatedAmount.totalAmount.toString()).eq(mintedAmount1.add(mintedAmount2).toString());
        expect(res2.event.userUpdatedRewardPool.updates[0].updatedUserRewardPools[1].tokenAllocatedAmount.numRecords).eq(2);
    });
});
