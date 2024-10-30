import { BN } from "@coral-xyz/anchor";
import { expect } from "chai";
import { step } from "mocha-steps";
import { restakingPlayground } from "../restaking";
import { getLogger } from '../../tools/lib';

const { logger, LOG_PAD_SMALL, LOG_PAD_LARGE } = getLogger("restaking");

describe("operate", async () => {
    const restaking = await restakingPlayground;

    step("fund operation: staking, normalization, restaking", async function () {
        const fragSOLFund0 = await restaking.getFragSOLFundAccount();
        const fragSOLFundExecutionReservedAccountBalance0 = await restaking.getFragSOLFundExecutionReservedAccountBalance();
        const nSOLPool0 = await restaking.getNSOLTokenPoolAccount();
        const nSOLMint0 = await restaking.getNSOLTokenMint();
        logger.info(`[BEFORE] fundSupportedTokens=${fragSOLFund0.supportedTokens.map(v => v.operationReservedAmount.toString()).join(', ')}, fundSolOperationReservedAmount=${fragSOLFund0.solOperationReservedAmount}, fundExecutionReservedAmount=${fragSOLFundExecutionReservedAccountBalance0}`);
        logger.info(`[BEFORE] nSOLSupportedTokens=${nSOLPool0.supportedTokens.map(v => v.lockedAmount.toString()).join(', ')}, nSOLSupply=${nSOLPool0.normalizedTokenMint}, fundExecutionReservedAmount=${fragSOLFundExecutionReservedAccountBalance0}`);
        expect(fragSOLFundExecutionReservedAccountBalance0.toString()).eq('0', 'execution reserved should be zero before operation');

        const jitoSolSupportedTokenAccount = restaking.knownAddress.fragSOLSupportedTokenAccount("jitoSOL");
        const jitoSolBalance0 = await restaking.connection.getTokenAccountBalance(jitoSolSupportedTokenAccount, "confirmed");
        expect(fragSOLFund0.supportedTokens.some(s => s.operationReservedAmount.toString() == jitoSolBalance0.value.amount.toString())).eq(true, 'supported ATA balance should be equal');

        const { fragSOLFundExecutionReservedAccountBalance: fragSOLFundExecutionReservedAccountBalance1, fragSOLFund: fragSOLFund1 } = await restaking.runOperatorRun(restaking.keychain.getKeypair('ADMIN'));
        logger.info(`[AFTER] supported_tokens=${fragSOLFund1.supportedTokens.map(v => v.operationReservedAmount.toString()).join(', ')}, solOperationReservedAmount=${fragSOLFund1.solOperationReservedAmount}, executionReservedAmount=${fragSOLFundExecutionReservedAccountBalance1}`);
        expect(fragSOLFundExecutionReservedAccountBalance1.toString()).eq('0', 'execution reserved should be zero after operation');

        const executedAmount = fragSOLFund0.solOperationReservedAmount.sub(fragSOLFund1.solOperationReservedAmount);
        expect(executedAmount.gt(new BN(0))).eq(true, 'executed amount should be greater than zero');

        const jitoSolBalance1 = await restaking.connection.getTokenAccountBalance(jitoSolSupportedTokenAccount, "confirmed");
        expect(fragSOLFund1.supportedTokens.some(s => s.operationReservedAmount.toString() == jitoSolBalance1.value.amount.toString())).eq(true, 'blabla3');
        const mintedTokenAmount = new BN(jitoSolBalance1.value.amount).sub(new BN(jitoSolBalance0.value.amount));
        expect(mintedTokenAmount.gt(new BN(0))).eq(true, 'minted supported tokens should be greater than zero');
        expect(mintedTokenAmount.gt(executedAmount.div(new BN(2)))).eq(true, 'minted supported tokens should be not too less than staked sol amount');

        expect(fragSOLFund1.solOperationReservedAmount.toString()).eq('0', "fund account's solOperationReservedAmount should be 0");
    });
});
