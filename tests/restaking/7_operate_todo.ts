import {BN, web3} from "@coral-xyz/anchor";
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";
import {getLogger} from '../../tools/lib';

const {logger, LOG_PAD_SMALL, LOG_PAD_LARGE} = getLogger("restaking");

module.exports = (i: number) => describe(`operate#TODO${i}`, async () => {
    const restaking = await restakingPlayground;
    const user1 = restaking.keychain.getKeypair('MOCK_USER1');

    step("deposit sol and tokens & request withdraw for most of them", async () => {
        await Promise.all([
            restaking.tryAirdrop(user1.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(200)),
            restaking.tryAirdropSupportedTokens(user1.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(200)),
        ]);
        await restaking.runUserDepositSOL(user1, new BN(web3.LAMPORTS_PER_SOL).muln(200));
        await restaking.runUserDepositSupportedToken(user1, 'bSOL', new BN(web3.LAMPORTS_PER_SOL).muln(50));
        const res0 = await restaking.runUserDepositSupportedToken(user1, 'mSOL', new BN(web3.LAMPORTS_PER_SOL).muln(50));
        const res1 = await restaking.getUserFragSOLFundAccount(user1.publicKey);

        // turn on withdrawable for supported tokens
        for (const supportedToken of res0.fragSOLFund.supportedTokens.slice(0, res0.fragSOLFund.numSupportedTokens)) {
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
        }
        const res2 = await restaking.runUserRequestWithdrawal(user1, res1.receiptTokenAmount.divn(2));
        const res3 = await restaking.runUserRequestWithdrawal(user1, res1.receiptTokenAmount.divn(4), restaking.getConstantAsPublicKey('mainnetBsolMintAddress'));
        const res4 = await restaking.runUserRequestWithdrawal(user1, res1.receiptTokenAmount.divn(4), restaking.getConstantAsPublicKey('mainnetMsolMintAddress'));
    });

    step("fund operation for a full cycle (ncn_epoch = 256 slot)", async () => {
        await restaking.runOperatorFundCommands();
    });

    step("fund operation for the next cycle after an epoch shift", async () => {
        logger.info('waiting for an epoch shift...')
        await restaking.sleepUntil(254); // wait until just before the end of the current epoch (instead of 256) to make more complex scenario
        await restaking.runOperatorFundCommands();
    });
});
