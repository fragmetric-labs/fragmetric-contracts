import * as anchor from "@coral-xyz/anchor";
import {BN, web3} from "@coral-xyz/anchor";
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";
import { RestakingPlayground } from "../../tools/restaking/playground";

describe("deposit_sol", async () => {
    const restaking = await restakingPlayground as RestakingPlayground;
    const user1 = restaking.keychain.getKeypair('MOCK_USER1');
    const user2 = restaking.keychain.getKeypair('MOCK_USER2');

    const amount = new BN(10 * web3.LAMPORTS_PER_SOL);

    step("try airdrop SOL to mock accounts", async function () {
        await Promise.all([
            restaking.tryAirdrop(user1.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(100)),
            restaking.tryAirdrop(user2.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(100)),
        ]);

        await restaking.sleep(1); // ...block hash not found?
    });

    step("user1 deposits SOL", async function () {
        const [
            fragSOLFund0,
            fragSOLFundReserveAccountBalance0,
        ] = await Promise.all([
            restaking.getFragSOLFundAccount(),
            restaking.getFragSOLFundReserveAccountBalance(),
        ]);
        // expect(fragSOLFund0.sol.oneReceiptTokenAsSol.toNumber()).greaterThan(0, 'fragSOL price is not zero');
        expect(fragSOLFund0.sol.operationReservedAmount.toString()).eq(fragSOLFundReserveAccountBalance0.toString(), 'fund records correct amount of SOL reserved');
        const [
            userFundAccount0,
            userRewardAccount0,
        ] = await Promise.all([
            restaking.getUserFragSOLFundAccount(user1.publicKey).catch(v => null),
            restaking.getUserFragSOLRewardAccount(user1.publicKey).catch(v => null)
        ]);

        const res1 = await restaking.runUserDepositSOL(user1, amount, null);

        expect(res1.event.userDepositedToFund.supportedTokenMint).eq(null);
        expect(res1.fragSOLFundReserveAccountBalance.sub(fragSOLFundReserveAccountBalance0).toString()).eq(amount.toString(), 'SOL is transferred to fund reserve account');
        expect(res1.fragSOLFund.sol.operationReservedAmount.sub(fragSOLFund0.sol.operationReservedAmount).toString()).eq(amount.toString(), 'fund records correct amount of deposited SOL');

        expect(res1.fragSOLUserFund.user.toString()).eq(user1.publicKey.toString(), 'user check');
        expect(res1.fragSOLUserFund.receiptTokenAmount.toString()).eq(res1.fragSOLUserTokenAccount.amount.toString(), 'user fund records correct amount of minted fragSOL');

        const mintedAmount = res1.fragSOLUserFund.receiptTokenAmount.sub(userFundAccount0?.receiptTokenAmount ?? new BN(0));
        expect(mintedAmount.toString()).eq(amount.toString());

        const [
            fragSOLFund2,
            fragSOLFundReserveAccountBalance2,
        ] = await Promise.all([
            restaking.getFragSOLFundAccount(),
            restaking.getFragSOLFundReserveAccountBalance(),
        ]);

        expect(fragSOLFund2.oneReceiptTokenAsSol.toNumber()).eq(web3.LAMPORTS_PER_SOL);
        expect(fragSOLFundReserveAccountBalance2.sub(fragSOLFundReserveAccountBalance0).toString()).eq(amount.toString(), '11');
    });

    step("donates SOL and price changed", async () => {
        const { fragSOLFund } = await restaking.runOperatorDonateSOLToFund(restaking.wallet, amount);

        expect(fragSOLFund.oneReceiptTokenAsSol.toNumber()).eq(web3.LAMPORTS_PER_SOL * 2);
    });

    step("user2 deposits SOL", async () => {
        const [
            fragSOLFund0,
            fragSOLFundReserveAccountBalance0,
        ] = await Promise.all([
            restaking.getFragSOLFundAccount(),
            restaking.getFragSOLFundReserveAccountBalance(),
        ]);
        const [
            userFundAccount0,
            userRewardAccount0,
        ] = await Promise.all([
            restaking.getUserFragSOLFundAccount(user2.publicKey).catch(v => null),
            restaking.getUserFragSOLRewardAccount(user2.publicKey).catch(v => null)
        ]);

        const res1 = await restaking.runUserDepositSOL(user2, amount, null);

        expect(res1.event.userDepositedToFund.supportedTokenMint).eq(null);
        expect(res1.fragSOLFundReserveAccountBalance.sub(fragSOLFundReserveAccountBalance0).toString()).eq(amount.toString(), 'SOL is transferred to fund reserve account');
        expect(res1.fragSOLFund.sol.operationReservedAmount.sub(fragSOLFund0.sol.operationReservedAmount).toString()).eq(amount.toString(), 'fund records correct amount of deposited SOL');

        expect(res1.fragSOLUserFund.user.toString()).eq(user2.publicKey.toString(), 'user check');
        expect(res1.fragSOLUserFund.receiptTokenAmount.toString()).eq(res1.fragSOLUserTokenAccount.amount.toString(), 'user fund records correct amount of minted fragSOL');

        const mintedAmount = res1.fragSOLUserFund.receiptTokenAmount.sub(userFundAccount0?.receiptTokenAmount ?? new BN(0));
        expect(mintedAmount.toString()).eq(amount.divn(2).toString());

        const [
            fragSOLFund2,
            fragSOLFundReserveAccountBalance2,
        ] = await Promise.all([
            restaking.getFragSOLFundAccount(),
            restaking.getFragSOLFundReserveAccountBalance(),
        ]);

        expect(fragSOLFund2.oneReceiptTokenAsSol.toNumber()).eq(web3.LAMPORTS_PER_SOL * 2);
        expect(fragSOLFundReserveAccountBalance2.sub(fragSOLFundReserveAccountBalance0).toString()).eq(amount.toString(), '11');
    })
});
