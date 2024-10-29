import * as anchor from "@coral-xyz/anchor";
import {BN} from '@coral-xyz/anchor';
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";

describe("deposit_token", async () => {
    const restaking = await restakingPlayground;
    const user3 = restaking.keychain.getKeypair('MOCK_USER3');
    const user4 = restaking.keychain.getKeypair('MOCK_USER4');

    step("try airdrop SOL and supported tokens to mock accounts", async function () {
        await Promise.all([
            restaking.tryAirdrop(user3.publicKey, 100),
            restaking.tryAirdrop(user4.publicKey, 100),
        ]);

        await restaking.sleep(1); // ...block hash not found?

        await Promise.all([
            restaking.tryAirdropSupportedTokens(user3.publicKey, 100_000),
            restaking.tryAirdropSupportedTokens(user4.publicKey, 100_000),
        ]);

        await restaking.sleep(1);
    });

    step("user3 deposits supported token without metadata to mint fragSOL", async function () {
        const res0 = await restaking.runOperatorUpdatePrices();
        expect(res0.event.operatorUpdatedFundPrice.fundAccount.oneReceiptTokenAsSol.toNumber()).greaterThan(0);
        expect(res0.fragSOLFundBalance.toNumber()).greaterThan(0);
        const fragSOLPrice0 = res0.event.operatorUpdatedFundPrice.fundAccount.oneReceiptTokenAsSol;

        const decimals = 10 ** 9;
        const amount = new BN(10 * decimals);
        const symbol = 'bSOL';
        const initialTokenAmount = new BN((await restaking.getUserSupportedTokenAccount(user3.publicKey, symbol)).amount.toString());
        const res1 = await restaking.runUserDepositSupportedToken(user3, symbol, amount, null);

        expect(new BN(res1.userSupportedTokenAccount.amount.toString()).toString()).eq(new BN(initialTokenAmount).sub(amount).toString());
        expect(res1.fragSOLFund.supportedTokens[0].operationReservedAmount.toString()).eq(amount.toString());
        const fragSOLPrice1 = res1.event.userDepositedSupportedTokenToFund.fundAccount.oneReceiptTokenAsSol;

        expect(fragSOLPrice0.toString()).eq(fragSOLPrice1.toString()); // price is consistent upon deposits

        const tokenPrice = res1.fragSOLFund.supportedTokens[0].oneTokenAsSol;
        expect(tokenPrice.mul(amount).div(fragSOLPrice1).toString()).eq(res1.fragSOLUserTokenAccount.amount.toString()); // proper amount minted?

        expect(res1.event.userDepositedSupportedTokenToFund.walletProvider).null;
        expect(res1.event.userDepositedSupportedTokenToFund.contributionAccrualRate).null;
        expect(res1.event.userUpdatedRewardPool.updatedUserRewardAccountAddresses.length).eq(1);
    });

    step("user3 fails to deposit too many tokens", async function () {
        const decimals = 10 ** 9;
        const amount = new BN((10 ** 4) * decimals);
        const symbol = 'bSOL';
        await expect(restaking.runUserDepositSupportedToken(user3, symbol, amount, null)).rejectedWith('FundExceededTokenCapacityAmountError');
    });

    step("user4 deposits supported token with metadata to mint fragSOL", async function () {
        const res0 = await restaking.runOperatorUpdatePrices();

        const symbol = 'bSOL';
        const decimals = 10 ** 9;
        const amount1 = new BN(6 * decimals);
        const currentTimestamp = new BN(Math.floor(Date.now() / 1000));
        const depositMetadata1 = restaking.asType<'depositMetadata'>({
            walletProvider: "BACKPACK",
            contributionAccrualRate: 130,
            expiredAt: currentTimestamp,
        });
        const res1 = await restaking.runUserDepositSupportedToken(user4, symbol, amount1, depositMetadata1);
        const mintedAmount1 = res1.event.userDepositedSupportedTokenToFund.mintedReceiptTokenAmount;

        expect(res1.fragSOLFund.supportedTokens[0].operationReservedAmount.sub(res0.fragSOLFund.supportedTokens[0].operationReservedAmount).toString()).eq(amount1.toString());
        expect(res1.event.userDepositedSupportedTokenToFund.walletProvider).eq(depositMetadata1.walletProvider);
        expect(res1.event.userDepositedSupportedTokenToFund.contributionAccrualRate.toString()).eq(depositMetadata1.contributionAccrualRate.toString());
        expect(res1.event.userDepositedSupportedTokenToFund.userFundAccount.receiptTokenAmount.toString()).eq(res1.fragSOLUserTokenAccount.amount.toString())

        expect(res1.event.userUpdatedRewardPool.updatedUserRewardAccountAddresses.length).eq(1);
        const userRewardAccount1 = await restaking.getUserFragSOLRewardAccount(user4.publicKey);
        expect(userRewardAccount1.userRewardPools1[0].tokenAllocatedAmount.totalAmount.toString()).eq(mintedAmount1.toString());
        expect(userRewardAccount1.userRewardPools1[1].tokenAllocatedAmount.totalAmount.toString()).eq(mintedAmount1.toString());

        const amount2 = new BN(4 * decimals);
        const depositMetadata2 = restaking.asType<'depositMetadata'>({
            walletProvider: "FRONTPACK",
            contributionAccrualRate: 110,
            expiredAt: currentTimestamp,
        });
        const res2 = await restaking.runUserDepositSupportedToken(user4, symbol, amount2, depositMetadata2);
        const mintedAmount2 = res2.event.userDepositedSupportedTokenToFund.mintedReceiptTokenAmount;

        expect(res2.fragSOLFund.supportedTokens[0].operationReservedAmount.sub(res1.fragSOLFund.supportedTokens[0].operationReservedAmount).toString(), 'added reserved token amount').eq(amount2.toString(), 'deposited token amount');
        expect(res2.event.userDepositedSupportedTokenToFund.walletProvider).eq(depositMetadata2.walletProvider);
        expect(res2.event.userDepositedSupportedTokenToFund.contributionAccrualRate.toString()).eq(depositMetadata2.contributionAccrualRate.toString());
        expect(res2.event.userDepositedSupportedTokenToFund.userFundAccount.receiptTokenAmount.toString()).eq(res2.fragSOLUserTokenAccount.amount.toString())

        expect(res2.event.userUpdatedRewardPool.updatedUserRewardAccountAddresses.length).eq(1);
        const userRewardAccount2 = await restaking.getUserFragSOLRewardAccount(user4.publicKey);
        expect(userRewardAccount2.userRewardPools1[0].tokenAllocatedAmount.totalAmount.toString(), 'total allocated amount').eq(mintedAmount1.add(mintedAmount2).toString(), 'minted fragSOL amount');
        expect(userRewardAccount2.userRewardPools1[0].tokenAllocatedAmount.numRecords).eq(1);
        expect(userRewardAccount2.userRewardPools1[1].tokenAllocatedAmount.totalAmount.toString()).eq(mintedAmount1.add(mintedAmount2).toString());
        expect(userRewardAccount2.userRewardPools1[1].tokenAllocatedAmount.numRecords).eq(2);
    });
});
