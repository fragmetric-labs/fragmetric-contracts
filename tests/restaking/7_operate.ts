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
        await restaking.runUserDepositSOL(user1, new BN(web3.LAMPORTS_PER_SOL).muln(50));
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
        let quarter = res1.receiptTokenAmount.divn(4);
        await restaking.runUserRequestWithdrawal(user1, quarter, restaking.getConstantAsPublicKey('mainnetBsolMintAddress'));
        await restaking.runUserRequestWithdrawal(user1, quarter, restaking.getConstantAsPublicKey('mainnetMsolMintAddress'));
        await restaking.runOperatorFundCommands();
        logger.info('waiting... (1 epoch = 64 slots)');
        await restaking.sleepUntil(192);

        await restaking.runUserWithdraw(user1, restaking.getConstantAsPublicKey('mainnetBsolMintAddress'), new BN(1));
        await restaking.runUserWithdraw(user1, restaking.getConstantAsPublicKey('mainnetMsolMintAddress'), new BN(1));
        const res2 = await restaking.getUserFragSOLFundAccount(user1.publicKey);
        await restaking.runUserRequestWithdrawal(user1, res2.receiptTokenAmount);
        await restaking.runOperatorFundCommands(); // here a unrestaking request made

        logger.info('waiting...');
        await restaking.sleepUntil(320); // wait for more than one epoch
        await restaking.runOperatorFundCommands(); // the unrestaking request should be claimable on this cycle
        await restaking.runOperatorFundCommands(); // one more cycle to denormalize and unstake tokens

        logger.info('waiting...');
        await restaking.sleepUntil(440); // wait for more than one epoch
        await restaking.runOperatorFundCommands(); // one more cycle to claim unstaked tokens and proceed the last withdrawal batch
        await restaking.runUserWithdraw(user1, null, new BN(1));
    });
});
