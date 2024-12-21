import * as anchor from "@coral-xyz/anchor";
import {BN, web3} from "@coral-xyz/anchor";
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";

module.exports = (i: number) => describe(`deposit_sol#${i}`, async () => {
    const restaking = await restakingPlayground;
    const user1 = restaking.keychain.getKeypair('MOCK_USER1');
    const user2 = restaking.keychain.getKeypair('MOCK_USER2');

    step("try airdrop SOL to mock accounts", async function () {
        await Promise.all([
            restaking.tryAirdrop(user1.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(100)),
            restaking.tryAirdrop(user2.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(100)),
        ]);

        await restaking.sleep(1); // ...block hash not found?
    });

    step("user1 deposits SOL without metadata to mint fragSOL", async function () {
        const [
            fragSOLFund0,
            fragSOLFundReserveAccountBalance0,
        ] = await Promise.all([
            restaking.getFragSOLFundAccount(),
            restaking.getFragSOLFundReserveAccountBalance(),
        ]);
        expect(fragSOLFund0.oneReceiptTokenAsSol.toNumber()).greaterThan(0, 'fragSOL price is not zero');
        expect(fragSOLFund0.sol.operationReservedAmount.toString()).eq(fragSOLFundReserveAccountBalance0.toString(), 'fund records correct amount of SOL reserved');
        const [
            userFundAccount0,
            userRewardAccount0,
        ] = await Promise.all([
            restaking.getUserFragSOLFundAccount(user1.publicKey).catch(v => null),
            restaking.getUserFragSOLRewardAccount(user1.publicKey).catch(v => null)
        ]);

        const amount = new BN(10 * anchor.web3.LAMPORTS_PER_SOL);
        const res1 = await restaking.runUserDepositSOL(user1, amount, null);

        // TODO/v0.4: do deposit test also with pricing changes like below ... currently it break other tests
        // await restaking.tryAirdrop(restaking.keychain.wallet.publicKey, new BN(10**9));
        // const resX1 = await restaking.runOperatorDonateSOLToFund(restaking.keychain.wallet, new BN(10**9));
        // await restaking.tryAirdropSupportedTokens(restaking.keychain.wallet.publicKey, new BN(10**9 * 2));
        // const resX2 = await restaking.runOperatorDonateSupportedTokenToFund(restaking.keychain.wallet, 'bSOL', new BN(10**9));
        // const resX3 = await restaking.runUserDepositSOL(user1, amount, null);
        // expect(resX3.event.userDepositedToFund.mintedReceiptTokenAmount.toNumber()).lt(amount.toNumber());

        expect(res1.event.userDepositedToFund.supportedTokenMint).eq(null);
        expect(res1.fragSOLFundReserveAccountBalance.sub(fragSOLFundReserveAccountBalance0).toString()).eq(amount.toString(), 'SOL is transferred to fund reserve account');
        expect(res1.fragSOLFund.sol.operationReservedAmount.sub(fragSOLFund0.sol.operationReservedAmount).toString()).eq(amount.toString(), 'fund records correct amount of deposited SOL');

        expect(res1.fragSOLUserFund.user.toString()).eq(user1.publicKey.toString(), 'user check');
        expect(res1.fragSOLUserFund.receiptTokenAmount.toString()).eq(res1.fragSOLUserTokenAccount.amount.toString(), 'user fund records correct amount of minted fragSOL');

        expect(res1.event.userDepositedToFund.walletProvider).null;
        expect(res1.event.userDepositedToFund.contributionAccrualRate).null;
        // expect(res1.fragSOLUserFund.receiptTokenAmount.toString()).eq(res1.fragSOLUserTokenAccount.amount.toString(), 'fragSOL mint amount in event is valid');
        const mintedAmount = res1.fragSOLUserFund.receiptTokenAmount.sub(userFundAccount0?.receiptTokenAmount ?? new BN(0));

        expect(res1.event.userDepositedToFund.updatedUserRewardAccounts.length).eq(1, 'user reward account is in event');
        const userRewardAccount1 = await restaking.getUserFragSOLRewardAccount(user1.publicKey);
        expect(userRewardAccount1.userRewardPools1[0].tokenAllocatedAmount.totalAmount.sub(userRewardAccount0?.userRewardPools1[0].tokenAllocatedAmount.totalAmount ?? new BN(0)).toString()).eq(mintedAmount.toString(), 'user reward account updated base pool');
        expect(userRewardAccount1.userRewardPools1[1].tokenAllocatedAmount.totalAmount.sub(userRewardAccount0?.userRewardPools1[1].tokenAllocatedAmount.totalAmount ?? new BN(0)).toString()).eq(mintedAmount.toString(), 'user reward account updated bonus pool');

        const [
            // fragSOLFund2,
            fragSOLFundReserveAccountBalance2,
        ] = await Promise.all([
            // restaking.getFragSOLFundAccount(),
            restaking.getFragSOLFundReserveAccountBalance(),
        ]);

        // expect(res2.event.operatorUpdatedFundPrice.fundAccount.oneReceiptTokenAsSol.toString()).eq((mintedAmount.div(new BN(res1.fragSOLUserTokenAccount.amount.toString())).mul(new BN(10 ** restaking.fragSOLDecimals))).toString(), '11');
        expect(fragSOLFundReserveAccountBalance2.sub(fragSOLFundReserveAccountBalance0).toString()).eq(amount.toString(), '11');
    });

    step("user2 deposits SOL with metadata to mint fragSOL", async function () {
        const [
            fragSOLFundReserveAccountBalance0,
            userFundAccount0,
            userRewardAccount0,
        ] = await Promise.all([
            restaking.getFragSOLFundReserveAccountBalance(),
            restaking.getUserFragSOLFundAccount(user2.publicKey).catch(v => null),
            restaking.getUserFragSOLRewardAccount(user2.publicKey).catch(v => null),
        ]);

        const amount1 = new BN(6 * anchor.web3.LAMPORTS_PER_SOL);
        const currentTimestamp = new BN(Math.floor(Date.now() / 1000));
        const depositMetadata1 = restaking.asType<'depositMetadata'>({
            user: user2.publicKey,
            walletProvider: "BACKPACK",
            contributionAccrualRate: 130,
            expiredAt: currentTimestamp,
        });
        const res1 = await restaking.runUserDepositSOL(user2, amount1, depositMetadata1);

        expect(res1.fragSOLFundReserveAccountBalance.sub(fragSOLFundReserveAccountBalance0).toString()).eq(amount1.toString(), 'SOL is transferred to fund reserve account');
        expect(res1.event.userDepositedToFund.walletProvider).eq(depositMetadata1.walletProvider, 'wallet provider is correct');
        expect(res1.event.userDepositedToFund.contributionAccrualRate.toString()).eq(depositMetadata1.contributionAccrualRate.toString(), 'contribution accrual rate is correct');
        // expect(res1.fragSOLUserFund.receiptTokenAmount.toString()).eq(res1.fragSOLUserTokenAccount.amount.toString(), 'fragSOL mint amount in event is valid');
        const mintedAmount1 = res1.fragSOLUserFund.receiptTokenAmount.sub(userFundAccount0?.receiptTokenAmount ?? new BN(0));

        expect(res1.event.userDepositedToFund.updatedUserRewardAccounts.length).eq(1, 'user reward account is in event');
        const userRewardAccount1 = await restaking.getUserFragSOLRewardAccount(user2.publicKey);
        expect(userRewardAccount1.userRewardPools1[0].tokenAllocatedAmount.totalAmount.sub(userRewardAccount0?.userRewardPools1[0].tokenAllocatedAmount.totalAmount ?? new BN(0)).toString()).eq(mintedAmount1.toString(), 'user reward account updated base pool');
        expect(userRewardAccount1.userRewardPools1[1].tokenAllocatedAmount.totalAmount.sub(userRewardAccount0?.userRewardPools1[1].tokenAllocatedAmount.totalAmount ?? new BN(0)).toString()).eq(mintedAmount1.toString(), 'user reward account updated bonus pool');

        const amount2 = new BN(4 * anchor.web3.LAMPORTS_PER_SOL);
        const depositMetadata2 = restaking.asType<'depositMetadata'>({
            user: user2.publicKey,
            walletProvider: "FRONTPACK",
            contributionAccrualRate: 110,
            expiredAt: currentTimestamp,
        });
        const res2 = await restaking.runUserDepositSOL(user2, amount2, depositMetadata2);
        const mintedAmount2 = res2.fragSOLUserFund.receiptTokenAmount.sub(res1.fragSOLUserFund.receiptTokenAmount);

        expect(res2.fragSOLFundReserveAccountBalance.sub(res1.fragSOLFundReserveAccountBalance).toString()).eq(amount2.toString(), '8');
        expect(res2.event.userDepositedToFund.walletProvider).eq(depositMetadata2.walletProvider, '9');
        expect(res2.event.userDepositedToFund.contributionAccrualRate.toString()).eq(depositMetadata2.contributionAccrualRate.toString(), '10');
        expect(res2.fragSOLUserFund.receiptTokenAmount.toString()).eq(res2.fragSOLUserTokenAccount.amount.toString(), '11')

        expect(res2.event.userDepositedToFund.updatedUserRewardAccounts.length).eq(1, '12');
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
            user: user1.publicKey,
            walletProvider: "MYPACK",
            contributionAccrualRate: 200,
            expiredAt: currentTimestamp,
        });

        // invalid signer
        await expect(restaking.runUserDepositSOL(user2, amount1, depositMetadata1, user2)).rejectedWith('InvalidSignatureError');

        // invalid user key
        await expect(restaking.runUserDepositSOL(user2, amount1, depositMetadata1)).rejectedWith('RequireKeysEqViolated');
    });

    step("signature verification has to fail when after expiration", async function () {
        const amount1 = new BN(5 * anchor.web3.LAMPORTS_PER_SOL);
        const expirationTimestamp = new BN(Math.floor(Date.now() / 1000) - 5); // expired 2 sec ago
        const depositMetadata1 = restaking.asType<'depositMetadata'>({
            user: user2.publicKey,
            walletProvider: "BACKPACK",
            contributionAccrualRate: 130,
            expiredAt: expirationTimestamp,
        });
        await expect(restaking.runUserDepositSOL(user2, amount1, depositMetadata1)).rejectedWith('FundDepositMetadataSignatureExpiredError');
    });
});
