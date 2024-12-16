import {BN} from '@coral-xyz/anchor';
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";
import {getLogger} from "../../tools/lib";

const {logger} = getLogger('reward');

module.exports = (i: number) => describe(`deposit_token#${i}`, async () => {
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
        const [
            fragSOLFund0,
            fragSOLFundReserveAccountBalance0,
            fragSOLUserTokenAccount0,
        ] = await Promise.all([
            restaking.getFragSOLFundAccount(),
            restaking.getFragSOLFundReserveAccountBalance(),
            restaking.getUserFragSOLAccount(user3.publicKey).catch(v => null),
        ]);
        expect(fragSOLFund0.oneReceiptTokenAsSol.toNumber()).greaterThan(0, '1');
        expect(fragSOLFundReserveAccountBalance0.toString()).eq(fragSOLFund0.sol.operationReservedAmount.toString(), '2');
        const fragSOLPrice0 = fragSOLFund0.oneReceiptTokenAsSol;

        const decimals = 10 ** 9;
        const amount = new BN(10 * decimals);
        const symbol = 'bSOL';
        const initialTokenAmount = new BN((await restaking.getUserSupportedTokenAccount(user3.publicKey, symbol)).amount.toString());
        const res1 = await restaking.runUserDepositSupportedToken(user3, symbol, amount, null);

        expect(res1.event.userDepositedToFund.supportedTokenMint.toString()).eq(res1.fragSOLFund.supportedTokens[0].mint.toString());
        expect(new BN(res1.userSupportedTokenAccount.amount.toString()).toString()).eq(new BN(initialTokenAmount).sub(amount).toString(), '3');
        expect(res1.fragSOLFund.supportedTokens[0].token.operationReservedAmount.sub(fragSOLFund0.supportedTokens[0].token.operationReservedAmount).toString()).eq(amount.toString(), '4');
        const fragSOLPrice1 = res1.fragSOLFund.oneReceiptTokenAsSol;

        expect(fragSOLPrice0.toString()).eq(fragSOLPrice1.toString(), '5'); // price is consistent around deposits

        const tokenPrice = res1.fragSOLFund.supportedTokens[0].oneTokenAsSol;
        expect(tokenPrice.mul(amount).div(fragSOLPrice1).div(new BN(100)).toString())
            .eq(new BN(res1.fragSOLUserTokenAccount.amount.toString()).sub(new BN(fragSOLUserTokenAccount0?.amount.toString() ?? 0)).div(new BN(100)).toString(), '6'); // proper amount minted?

        expect(res1.event.userDepositedToFund.walletProvider).null;
        expect(res1.event.userDepositedToFund.contributionAccrualRate).null;
        expect(res1.event.userDepositedToFund.updatedUserRewardAccounts.length).eq(1, '7');
    });

    step("user3 fails to deposit too many tokens", async function () {
        const decimals = 10 ** 9;
        const amount = new BN((10 ** 4) * decimals);
        const symbol = 'bSOL';
        await expect(restaking.runUserDepositSupportedToken(user3, symbol, amount, null)).rejectedWith('FundExceededDepositCapacityAmountError');
    });

    step("user4 deposits supported token with metadata to mint fragSOL", async function () {
        const [
            fragSOLFund0,
            fragSOLFundReserveAccountBalance0,
            // fragSOLUserTokenAccount0,
            userRewardAccount0,
        ] = await Promise.all([
            restaking.getFragSOLFundAccount(),
            restaking.getFragSOLFundReserveAccountBalance(),
            // restaking.getUserFragSOLAccount(user3.publicKey).catch(v => null),
            restaking.getUserFragSOLRewardAccount(user4.publicKey).catch(v => null),
        ]);

        const symbol = 'mSOL';
        const decimals = 10 ** 9;
        const amount1 = new BN(6 * decimals);
        const currentTimestamp = new BN(Math.floor(Date.now() / 1000) + 5);
        const depositMetadata1 = restaking.asType<'depositMetadata'>({
            user: user4.publicKey,
            walletProvider: "BACKPACK",
            contributionAccrualRate: 130,
            expiredAt: currentTimestamp,
        });
        const res1 = await restaking.runUserDepositSupportedToken(user4, symbol, amount1, depositMetadata1);
        const mintedAmount1 = res1.event.userDepositedToFund.mintedReceiptTokenAmount;

        expect(res1.fragSOLFund.supportedTokens[2].token.operationReservedAmount.sub(fragSOLFund0.supportedTokens[2].token.operationReservedAmount).toString()).eq(amount1.toString(), '1');
        expect(res1.event.userDepositedToFund.walletProvider).eq(depositMetadata1.walletProvider, '2');
        expect(res1.event.userDepositedToFund.contributionAccrualRate.toString()).eq(depositMetadata1.contributionAccrualRate.toString(), '3');
        // expect(res1.fragSOLUserFund.receiptTokenAmount.toString()).eq(res1.fragSOLUserTokenAccount.amount.toString(), '4');

        expect(res1.event.userDepositedToFund.updatedUserRewardAccounts.length).eq(1, '5');
        const userRewardAccount1 = await restaking.getUserFragSOLRewardAccount(user4.publicKey);
        expect(userRewardAccount1.userRewardPools1[0].tokenAllocatedAmount.totalAmount.sub(userRewardAccount0?.userRewardPools1[0].tokenAllocatedAmount.totalAmount ?? new BN(0)).toString()).eq(mintedAmount1.toString(), '6');
        expect(userRewardAccount1.userRewardPools1[1].tokenAllocatedAmount.totalAmount.sub(userRewardAccount0?.userRewardPools1[1].tokenAllocatedAmount.totalAmount ?? new BN(0)).toString()).eq(mintedAmount1.toString(), '7');

        const amount2 = new BN(4 * decimals);
        const depositMetadata2 = restaking.asType<'depositMetadata'>({
            user: user4.publicKey,
            walletProvider: "FRONTPACK",
            contributionAccrualRate: 110,
            expiredAt: currentTimestamp,
        });
        const res2 = await restaking.runUserDepositSupportedToken(user4, symbol, amount2, depositMetadata2);
        const mintedAmount2 = res2.event.userDepositedToFund.mintedReceiptTokenAmount;

        expect(res2.fragSOLFund.supportedTokens[2].token.operationReservedAmount.sub(res1.fragSOLFund.supportedTokens[2].token.operationReservedAmount).toString(), 'added reserved token amount').eq(amount2.toString(), 'deposited token amount');
        expect(res2.event.userDepositedToFund.walletProvider).eq(depositMetadata2.walletProvider, '8');
        expect(res2.event.userDepositedToFund.contributionAccrualRate.toString()).eq(depositMetadata2.contributionAccrualRate.toString(), '9');
        // expect(res2.fragSOLUserFund.receiptTokenAmount.toString()).eq(res2.fragSOLUserTokenAccount.amount.toString(), '10');

        expect(res2.event.userDepositedToFund.updatedUserRewardAccounts.length).eq(1, '11');
        const userRewardAccount2 = await restaking.getUserFragSOLRewardAccount(user4.publicKey);
        expect(userRewardAccount2.userRewardPools1[0].tokenAllocatedAmount.totalAmount.sub(userRewardAccount1.userRewardPools1[0].tokenAllocatedAmount.totalAmount).toString(), 'total allocated amount').eq(mintedAmount2.toString(), 'minted fragSOL amount');
        expect(userRewardAccount2.userRewardPools1[0].tokenAllocatedAmount.numRecords).eq(1, '12');
        expect(userRewardAccount2.userRewardPools1[1].tokenAllocatedAmount.totalAmount.sub(userRewardAccount1.userRewardPools1[1].tokenAllocatedAmount.totalAmount).toString()).eq(mintedAmount2.toString(), '13');
        expect(userRewardAccount2.userRewardPools1[1].tokenAllocatedAmount.numRecords).eq(2, '14');
    });

    step('fund has correct token value data', async function () {
        const [
            fragSOLTokenMint,
            fragSOLFund,
            fragSOLFundReserveAccountBalance,
        ] = await Promise.all([
            restaking.getFragSOLTokenMint(),
            restaking.getFragSOLFundAccount(),
            restaking.getFragSOLFundReserveAccountBalance(),
        ]);

        // console.log({ fragSOLTokenMint, fragSOLFund, fragSOLFundReserveAccountBalance })

        expect(new BN(fragSOLTokenMint.supply.toString()).toString()).eq(fragSOLFund.receiptTokenValue.denominator.toString(), 'correct receipt token supply');
        expect(new BN(fragSOLTokenMint.supply.toString()).toString()).eq(fragSOLFund.receiptTokenSupplyAmount.toString(), 'correct receipt token supply');
        expect(new BN(fragSOLTokenMint.decimals.toString()).toString()).eq(fragSOLFund.receiptTokenDecimals.toString(), 'correct receipt token decimals');
        for (const asset of fragSOLFund.receiptTokenValue.numerator) {
            if (asset.discriminant == 1) { // SOL
                expect(asset.solAmount.toString()).eq(fragSOLFundReserveAccountBalance.toString(), 'correct fund reserved sol (wallet account)');
                expect(asset.solAmount.toString()).eq(fragSOLFund.sol.operationReservedAmount.toString(), 'correct fund reserved sol (data account)');
            } else if (asset.discriminant == 2) { // Token
                const supportedTokenAccount = await restaking.getFragSOLSupportedTokenAccountByMintAddress(asset.tokenMint);
                const supportedTokenData = fragSOLFund.supportedTokens.find(s => s.mint.toString() == asset.tokenMint.toString());
                const supportedTokenDataBalance = supportedTokenData.token.operationReservedAmount.add(supportedTokenData.token.operationReceivableAmount);
                logger.debug(`${asset.tokenMint} balance:`, asset.tokenAmount.toString(), supportedTokenDataBalance.toString());

                expect(asset.tokenAmount.toString()).eq(new BN(supportedTokenAccount.amount.toString()).toString(), `correct fund reserved supported token (token account, ${asset.tokenMint})`);
                expect(asset.tokenAmount.toString()).eq(supportedTokenDataBalance.toString(), `correct fund reserved supported token (data account, ${asset.tokenMint})`);
            }
        }
    })
});
