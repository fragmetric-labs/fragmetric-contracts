import { BN, web3 } from "@coral-xyz/anchor";
import { expect } from "chai";
import { step } from "mocha-steps";
import { restakingPlayground } from "../restaking";
import { getLogger } from '../../tools/lib';
import { RestakingPlayground } from "../../tools/restaking/playground";

const { logger, LOG_PAD_SMALL, LOG_PAD_LARGE } = getLogger("restaking");

module.exports = (i: number) => describe(`operate#${i}`, async () => {
    const restaking = await restakingPlayground as RestakingPlayground;
    const user1 = restaking.keychain.getKeypair('MOCK_USER1');

    const slotPerEpoch = 64;
    const epochToSlot = (epoch: number) => epoch * slotPerEpoch;

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

        logger.info(`waiting until epoch 2... (1 epoch = ${slotPerEpoch} slots)`);
        await restaking.sleepUntil(epochToSlot(2));
        logger.info('epoch 2: operator enqueue withdrawal - process withdrawal - stake - normalize - restake - delegate');
        await restaking.runOperatorFundCommands();

        await Promise.all([
            restaking.runUserWithdraw(user1, restaking.getConstantAsPublicKey('mainnetBsolMintAddress'), new BN(1)),
            restaking.runUserWithdraw(user1, restaking.getConstantAsPublicKey('mainnetMsolMintAddress'), new BN(1)),
            restaking.runUserRequestWithdrawal(user1, quarter),
        ]);

        logger.info('waiting until epoch 3...');
        await restaking.sleepUntil(epochToSlot(3));
        logger.info('epoch 3: operator enqueue withdrawal - request unrestake - undelegate');
        await restaking.runOperatorFundCommands(); // here a unrestaking request made

        logger.info('waiting until epoch 5...');
        await restaking.sleepUntil(epochToSlot(5)); // unrestaking takes for more than one epoch
        logger.info('epoch 5: operator claim unrestaked - denormalize - request unstake');
        await restaking.runOperatorFundCommands();

        const res2 = await restaking.getUserFragSOLFundAccount(user1.publicKey); // last withdrawal request
        await restaking.runUserRequestWithdrawal(user1, res2.receiptTokenAmount);

        logger.info('waiting until epoch 6...');
        await restaking.sleepUntil(epochToSlot(6));
        logger.info('epoch 6: operator enqueue withdrawal - claim unstaked - process withdrawal - request unrestake - undelegate');
        await restaking.runOperatorFundCommands();
        await restaking.runUserWithdraw(user1, null, new BN(1));

        logger.info('waiting until epoch 8...');
        await restaking.sleepUntil(epochToSlot(8)); // unrestaking takes for more than one epoch
        logger.info('epoch 8: operator claim unrestaked - denormalize - request unstake');
        await restaking.runOperatorFundCommands();

        // jitoSOL reward airdropped to vault but token account is not delegated.
        const rewardMetadata = restaking.rewardTokenMetadata['jitoSOL'];
        const vaultMetadata = restaking.restakingVaultMetadata['jitoNSOLVault'];
        await restaking.tryAirdropRewardToken(vaultMetadata.vault, 'jitoSOL', new BN(web3.LAMPORTS_PER_SOL * 30));

        logger.info('waiting until epoch 10...');
        await restaking.sleepUntil(epochToSlot(10)); // due to msol, unstaking takes for more than one epoch
        logger.info('epoch 10: operator claim unstaked - process withdrawal');
        await restaking.runOperatorFundCommands();
        await restaking.runUserWithdraw(user1, null, new BN(2));

        // delegate token account to fund account
        await restaking.runAdminDelegateJitoVaultTokenAccount(vaultMetadata.vault, rewardMetadata.mint);

        logger.info('waiting until epoch 11...');
        await restaking.sleepUntil(epochToSlot(11));
        logger.info('epoch 11: operator harvest reward');
        await restaking.runOperatorFundCommands();

        logger.info('try to initialize in same epoch');
        await restaking.runOperatorFundCommands({
            command: {
                initialize: {
                    0: {
                        state: {
                            newRestakingVaultUpdate: {},
                        }
                    }
                }
            },
            requiredAccounts: [],
        });
    });
});
