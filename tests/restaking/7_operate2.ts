import {BN, web3} from "@coral-xyz/anchor";
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";
import {getLogger} from '../../tools/lib';

const {logger, LOG_PAD_SMALL, LOG_PAD_LARGE} = getLogger("restaking");

module.exports = (i: number) => describe(`operate#${i}`, async () => {
    const restaking = await restakingPlayground;
    const user1 = restaking.keychain.getKeypair('MOCK_USER1');

    step("deposit sol and tokens & request withdraw for most of them", async () => {
        await Promise.all([
            restaking.tryAirdrop(user1.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(200)),
            restaking.tryAirdropSupportedTokens(user1.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(200)),
        ]);
        await restaking.runUserDepositSOL(user1, new BN(web3.LAMPORTS_PER_SOL).muln(1));
        await restaking.runUserDepositSupportedToken(user1, 'bSOL', new BN(web3.LAMPORTS_PER_SOL).muln(50));
        const res0 = await restaking.runUserDepositSupportedToken(user1, 'mSOL', new BN(web3.LAMPORTS_PER_SOL).muln(50));
        const res1 = await restaking.getUserFragSOLFundAccount(user1.publicKey);

        let quarter = res1.receiptTokenAmount.divn(4);
        await restaking.runUserRequestWithdrawal(user1, quarter, null);
        await restaking.runUserRequestWithdrawal(user1, quarter, null);

        logger.info('waiting... (1 epoch = 32 slots)');
        await restaking.sleepUntil(128);
        await restaking.runOperatorFundCommands();

        logger.info('waiting...');
        await restaking.sleepUntil(160);
        await restaking.runOperatorFundCommands();

        logger.info('waiting...');
        await restaking.sleepUntil(192);
        await restaking.runOperatorFundCommands();

        logger.info('waiting...');
        await restaking.sleepUntil(256);
        await restaking.runOperatorFundCommands();

        logger.info('waiting...');
        await restaking.sleepUntil(288);
        await restaking.runOperatorFundCommands();

        const res2 = await restaking.getUserFragSOLFundAccount(user1.publicKey); // last withdrawal request
        await Promise.all([
            restaking.runUserWithdraw(user1, null, new BN(1)),
            restaking.runUserWithdraw(user1, null, new BN(2)),
            restaking.runUserRequestWithdrawal(user1, res2.receiptTokenAmount, null),
        ]);

        logger.info('waiting...');
        await restaking.sleepUntil(320);
        await restaking.runOperatorFundCommands();

        logger.info('waiting...');
        await restaking.sleepUntil(352);
        await restaking.runOperatorFundCommands();

        logger.info('waiting...');
        await restaking.sleepUntil(384);
        await restaking.runOperatorFundCommands();

        logger.info('waiting...');
        await restaking.sleepUntil(416);
        await restaking.runOperatorFundCommands();

        logger.info('waiting...');
        await restaking.sleepUntil(448);
        await restaking.runOperatorFundCommands();

        await restaking.runUserWithdraw(user1, null, new BN(3));
    });
});
