import {BN, web3} from '@coral-xyz/anchor';
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";
import {RestakingPlayground} from "../../tools/restaking/jto_playground";

describe("withdraw jto", async () => {
    const restaking = await restakingPlayground as RestakingPlayground;
    const user1 = restaking.keychain.getKeypair('MOCK_USER1');

    step("deposit jto & request withdraw for most of them", async () => {
        await Promise.all([
            restaking.tryAirdrop(user1.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(200)),
            restaking.tryAirdropJTO(user1.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(200)),
        ]);
        // await restaking.runUserDepositSOL(user1, new BN(web3.LAMPORTS_PER_SOL).muln(50));
        await restaking.runUserDepositSupportedToken(user1, 'JTO', new BN(web3.LAMPORTS_PER_SOL).muln(50));
        const res0 = await restaking.runUserDepositSupportedToken(user1, 'JTO', new BN(web3.LAMPORTS_PER_SOL).muln(50));
        const res1 = await restaking.getUserFragJTOFundAccount(user1.publicKey);

        // turn on withdrawable for supported tokens
        for (const supportedToken of res0.fragJTOFund.supportedTokens.slice(0, res0.fragJTOFund.numSupportedTokens)) {
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
                            receiptTokenMint: restaking.knownAddress.fragJTOTokenMint,
                        })
                        .instruction(),
                ],
                signerNames: ['FUND_MANAGER'],
                events: ['fundManagerUpdatedFund'],
            });
        }
        let quarter = res1.receiptTokenAmount.divn(4);
        await restaking.runUserRequestWithdrawal(user1, quarter, restaking.getConstantAsPublicKey('mainnetJtoMintAddress'));
        await restaking.runUserRequestWithdrawal(user1, quarter, restaking.getConstantAsPublicKey('mainnetJtoMintAddress'));
        await restaking.runUserRequestWithdrawal(user1, res1.receiptTokenAmount.sub(quarter).sub(quarter), restaking.getConstantAsPublicKey('mainnetJtoMintAddress'));
    });

    step("fund operation for a full cycle (ncn_epoch = 256 slot)", async () => {
        await restaking.runOperatorFundCommands();
        await restaking.runUserWithdraw(user1, restaking.getConstantAsPublicKey('mainnetJtoMintAddress'), new BN(1));
        await restaking.runUserWithdraw(user1, restaking.getConstantAsPublicKey('mainnetJtoMintAddress'), new BN(2));
        await restaking.runUserWithdraw(user1, restaking.getConstantAsPublicKey('mainnetJtoMintAddress'), new BN(3));
    });
});
