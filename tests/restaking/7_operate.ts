import { BN } from "@coral-xyz/anchor";
import { expect } from "chai";
import { step } from "mocha-steps";
import { restakingPlayground } from "../restaking";
import { getLogger } from '../../tools/lib';

const { logger, LOG_PAD_SMALL, LOG_PAD_LARGE } = getLogger("restaking");

module.exports = (i: number) => describe(`operate#${i}`, async () => {
    const restaking = await restakingPlayground;

    step("fund operation: staking, normalization, restaking", async function () {
        const fragSOLFund0 = await restaking.getFragSOLFundAccount();
        const fragSOLFundExecutionReservedAccountBalance0 = await restaking.getFragSOLFundExecutionReservedAccountBalance();
        const nSOLPool0 = await restaking.getNSOLTokenPoolAccount();
        const nSOLMint0 = await restaking.getNSOLTokenMint();
        logger.info(`[BEFORE] fundSupportedTokens=${fragSOLFund0.supportedTokens.map(v => v.operationReservedAmount.toString()).join(', ')}, fundSolOperationReservedAmount=${fragSOLFund0.solOperationReservedAmount}, fundExecutionReservedAmount=${fragSOLFundExecutionReservedAccountBalance0}`);
        logger.info(`[BEFORE] nSOLSupportedTokens=${nSOLPool0.supportedTokens.map(v => v.lockedAmount.toString()).join(', ')}, nSOLSupply=${nSOLMint0.supply.toString()}`);
        expect(fragSOLFundExecutionReservedAccountBalance0.toString()).eq('0', 'execution reserved should be zero before operation');

        // TODO: currently staking sol to hard-coded LST, like localnet: jitoSOL
        const jitoSolSupportedTokenAccount = restaking.knownAddress.fragSOLSupportedTokenAccount("jitoSOL");
        const jitoSolBalance0 = await restaking.connection.getTokenAccountBalance(jitoSolSupportedTokenAccount, "confirmed");
        expect(fragSOLFund0.supportedTokens.some(s => s.operationReservedAmount.toString() == jitoSolBalance0.value.amount.toString())).eq(true, 'supported ATA balance should be equal');

        const { fragSOLFundExecutionReservedAccountBalance: fragSOLFundExecutionReservedAccountBalance1, fragSOLFund: fragSOLFund1 } = await restaking.runOperatorRun(restaking.keychain.getKeypair('ADMIN'));
        const nSOLPool1 = await restaking.getNSOLTokenPoolAccount();
        const nSOLJitoSOLBalance1 = await restaking.getNSOLSupportedTokenLockAccountBalance('jitoSOL');
        const nSOLMint1 = await restaking.getNSOLTokenMint();
        logger.info(`[AFTER] fundSupportedTokens=${fragSOLFund1.supportedTokens.map(v => v.operationReservedAmount.toString()).join(', ')}, fundSolOperationReservedAmount=${fragSOLFund1.solOperationReservedAmount}, fundExecutionReservedAmount=${fragSOLFundExecutionReservedAccountBalance1}`);
        logger.info(`[AFTER] nSOLSupportedTokens=${nSOLPool1.supportedTokens.map(v => v.lockedAmount.toString()).join(', ')}, nSOLSupply=${nSOLMint1.supply.toString()}`);
        expect(fragSOLFundExecutionReservedAccountBalance1.toString()).eq('0', 'execution reserved should be zero after operation');

        const stakedSOLAmount = fragSOLFund0.solOperationReservedAmount.sub(fragSOLFund1.solOperationReservedAmount);
        expect(stakedSOLAmount.gt(new BN(0))).eq(true, 'executed amount should be greater than zero');

        const jitoSolBalance1 = await restaking.connection.getTokenAccountBalance(jitoSolSupportedTokenAccount, "confirmed");
        expect(fragSOLFund1.supportedTokens.some(s => s.operationReservedAmount.toString() == jitoSolBalance1.value.amount.toString())).eq(true, 'blabla3');
        const mintedLSTAmount = new BN(jitoSolBalance1.value.amount).sub(new BN(jitoSolBalance0.value.amount));
        expect(nSOLJitoSOLBalance1.gt(new BN(0))).eq(true, "nSOL's supported tokens should be greater than zero");
        // expect(mintedLSTAmount.gt(stakedSOLAmount.div(new BN(2)))).eq(true, 'minted supported tokens should be not too less than staked sol amount');

        expect(fragSOLFund1.solOperationReservedAmount.toString()).eq('0', "fund account's solOperationReservedAmount should be 0");
    });
});
