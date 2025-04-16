import {BN, web3} from '@coral-xyz/anchor';
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";
import {RestakingPlayground} from "../../tools/restaking/jto_playground";
import {getLogger} from "../../tools/lib";

const {logger, LOG_PAD_SMALL, LOG_PAD_LARGE} = getLogger("restaking");

module.exports = (i: number) => describe(`operate#TODO${i}`, async () => {
    const restaking = await restakingPlayground as RestakingPlayground;
    const user1 = restaking.keychain.getKeypair('MOCK_USER1');

    const slotPerEpoch = 64;
    const epochToSlot = (epoch: number) => epoch * slotPerEpoch;

    step("deposit sol and tokens & request withdraw for most of them", async () => {
        await Promise.all([
            restaking.tryAirdrop(user1.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(200)),
            restaking.tryAirdropJTO(user1.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(200)),
        ]);
        await restaking.runUserDepositSupportedToken(user1, 'JTO', new BN(web3.LAMPORTS_PER_SOL).muln(100));
        await restaking.runUserDepositSupportedToken(user1, 'JTO', new BN(web3.LAMPORTS_PER_SOL).muln(50));
        await restaking.runUserDepositSupportedToken(user1, 'JTO', new BN(web3.LAMPORTS_PER_SOL).muln(50));
        const res1 = await restaking.getUserFragJTOFundAccount(user1.publicKey);

        let quarter = res1.receiptTokenAmount.divn(4);
        await restaking.runUserRequestWithdrawal(user1, quarter, restaking.getConstantAsPublicKey('mainnetJtoMintAddress'));
        await restaking.runUserRequestWithdrawal(user1, quarter, restaking.getConstantAsPublicKey('mainnetJtoMintAddress'));

        logger.info(`waiting until epoch 2... (1 epoch = ${slotPerEpoch} slots)`);
        await restaking.sleepUntil(epochToSlot(2));
        logger.info('epoch 2: operator enqueue withdrawal - process withdrawal restake');
        await restaking.runOperatorFundCommands();

        await Promise.all([
            restaking.runUserWithdraw(user1, restaking.getConstantAsPublicKey('mainnetJtoMintAddress'), new BN(1)),
            restaking.runUserWithdraw(user1, restaking.getConstantAsPublicKey('mainnetJtoMintAddress'), new BN(2)),
            restaking.runUserRequestWithdrawal(user1, quarter),
        ])

        logger.info('waiting until epoch 3...');
        await restaking.sleepUntil(epochToSlot(3));
        logger.info('epoch 3: operator enqueue withdrawal - request unrestake');
        await restaking.runOperatorFundCommands();

        const res2 = await restaking.getUserFragJTOFundAccount(user1.publicKey);
        await restaking.runUserRequestWithdrawal(user1, res2.receiptTokenAmount);

        // jitoSOL reward airdropped to vault but token account is not delegated.
        const rewardMetadata = restaking.rewardTokenMetadata['jitoSOL'];
        const vaultMetadata = restaking.restakingVaultMetadata['jitoJTOVault'];
        await restaking.tryAirdropRewardToken(vaultMetadata.vault, 'jitoSOL', new BN(web3.LAMPORTS_PER_SOL * 30));

        // TODO improve pricing accuracy and then add withdrawal back
        logger.info('waiting until epoch 5...');
        await restaking.sleepUntil(epochToSlot(5)); // unrestaking takes for more than one epoch
        // logger.info('epoch 5: operator enqueue withdrawal - claim unrestaked - process withdrawal - request unrestake');
        logger.info('epoch 5: operator enqueue withdrawal - claim unrestaked - request unrestake');
        await restaking.runOperatorFundCommands();
        // await restaking.runUserWithdraw(user1, restaking.getConstantAsPublicKey('mainnetJtoMintAddress'), new BN(3));

        // delegate token account to fund account
        await restaking.runFundManagerDelegateJitoVaultTokenAccount(vaultMetadata.vault, rewardMetadata.mint);

        logger.info('waiting until epoch 7...');
        await restaking.sleepUntil(epochToSlot(7)); // unrestaking takes for more than one epoch
        logger.info('epoch 7: operator claim unrestaked - process withdrawal - harvest reward - restake');
        await restaking.runOperatorFundCommands();
        await restaking.runUserWithdraw(user1, restaking.getConstantAsPublicKey('mainnetJtoMintAddress'), new BN(4));

        logger.info('try to initialize in same epoch');
        await restaking.runOperatorFundCommands({
            command: {
                initialize: {
                    0: {
                        state: {
                            new: {},
                        }
                    }
                }
            },
            requiredAccounts: [],
        });
    });
});
