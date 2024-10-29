import * as anchor from "@coral-xyz/anchor";
// @ts-ignore
import * as spl from "@solana/spl-token";
import { BN } from '@coral-xyz/anchor';
import { expect } from "chai";
import { step } from "mocha-steps";
import { restakingPlayground } from "../restaking";
import { getLogger } from '../../tools/lib';

const { logger, LOG_PAD_SMALL, LOG_PAD_LARGE } = getLogger("restaking");

describe("operator_spl_stake_pool", async () => {
    const restaking = await restakingPlayground;

    step("Move fund to operation reserve account AND deposit sol to Jito stake pool", async function () {
        const beforeFundAccountState = await restaking.getFragSOLFundAccount();
        console.log(`[BEFORE] solAccumulatedDepositAmount: ${beforeFundAccountState.solAccumulatedDepositAmount}, solOperationReservedAmount: ${beforeFundAccountState.solOperationReservedAmount}`);

        const operationReserveAccountAddress = restaking.getOperationReserveAccountAddress();
        logger.debug(`operationReserveAccountAddress`.padEnd(LOG_PAD_LARGE), operationReserveAccountAddress);

        let operationReserveAccountSolBalance = await restaking.connection.getBalance(operationReserveAccountAddress, { commitment: "confirmed" });

        logger.debug(`[BEFORE] operation reserve account sol balance`.padEnd(LOG_PAD_LARGE), operationReserveAccountSolBalance);

        const [jitoSolSupporteTokenAccount] = restaking.knownAddress.fragSOLSupportedTokenAccount("jitoSOL");
        let jitoSolDepositedAmount = await restaking.connection.getTokenAccountBalance(jitoSolSupporteTokenAccount, "confirmed");

        logger.debug(`[BEFORE] jitoSol supported token account balance`.padEnd(LOG_PAD_LARGE), jitoSolDepositedAmount.value);

        const depositSolAmount = beforeFundAccountState.solOperationReservedAmount;

        const res0 = await restaking.runOperatorDepositSolToSplStakePool(restaking.wallet, depositSolAmount, restaking.supportedTokenMetadata.jitoSOL.mint, spl.TOKEN_PROGRAM_ID);

        logger.debug(`[AFTER] solOperationReservedAmount at fund account`.padEnd(LOG_PAD_LARGE), res0.fragSOLFund.solOperationReservedAmount.toString());

        operationReserveAccountSolBalance = await restaking.connection.getBalance(operationReserveAccountAddress, { commitment: "confirmed" });
        jitoSolDepositedAmount = await restaking.connection.getTokenAccountBalance(jitoSolSupporteTokenAccount, "confirmed");

        logger.debug(`[AFTER] operation reserve account sol balance`.padEnd(LOG_PAD_LARGE), operationReserveAccountSolBalance.toString());
        logger.debug(`[AFTER] jitoSol supported token account balance`.padEnd(LOG_PAD_LARGE), jitoSolDepositedAmount.value);

        expect(res0.fragSOLFund.solOperationReservedAmount.toString()).eq('0', "fund account's solOperationReservedAmount should be 0");
    });
});
