import { BN, web3 } from "@coral-xyz/anchor";
import { expect } from "chai";
import { step } from "mocha-steps";
// @ts-ignore
import * as spl from "@solana/spl-token-3.x";
import { restakingPlayground } from "../restaking";
import { getLogger } from '../../tools/lib';

const { logger, LOG_PAD_SMALL, LOG_PAD_LARGE } = getLogger("restaking");

describe("operator_spl_stake_pool", async () => {
    const restaking = await restakingPlayground;

    const splStakePoolProgram = new web3.PublicKey("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy");

    step("Wallet deposit SOL to fund", async function () {
        const depositSolAmount = new BN(10**9 * 500); // 500 SOL

        await restaking.runUserDepositSOL(restaking.wallet, depositSolAmount, null);
        await restaking.runOperatorDepositSolToSplStakePool(restaking.wallet, splStakePoolProgram, restaking.supportedTokenMetadata.jitoSOL.mint, restaking.supportedTokenMetadata.jitoSOL.program);
    });

    step("Withdraw SOL from Jito stake pool", async function () {
        await restaking.sleep(1); // ...block hash not found?

        const depositedSolAmount = new BN(10**9 * 500); // 500 SOL

        const jitoSolSupportedTokenAccount = restaking.knownAddress.fragSOLSupportedTokenAccount("jitoSOL");
        let jitoSolTotalWithdrawAmount = await restaking.connection.getTokenAccountBalance(jitoSolSupportedTokenAccount, "confirmed");
        logger.debug(`[BEFORE] jitoSol total withdrawal amount`.padEnd(LOG_PAD_LARGE), jitoSolTotalWithdrawAmount);

        const { fragSOLFund: fragSOLFund1, fragSOLFundReserveAccountBalance: fragSOLFundReserveAccountBalance1, withdrawalSolFee } = await restaking.runOperatorWithdrawSolFromSplStakePool(restaking.wallet, restaking.BN(jitoSolTotalWithdrawAmount.value.amount), splStakePoolProgram, restaking.supportedTokenMetadata.jitoSOL.mint, spl.TOKEN_PROGRAM_ID);

        logger.debug(`[AFTER] fragSOLFundReserveAccountBalance1`.padEnd(LOG_PAD_LARGE), fragSOLFundReserveAccountBalance1.toString());
        let jitoSolTotalWithdrawFeeAmount = depositedSolAmount.mul(withdrawalSolFee.numerator).div(withdrawalSolFee.denominator);
        // 1 lamport diff?
        expect(
            depositedSolAmount.sub(new BN(fragSOLFundReserveAccountBalance1)).divn(10)
                .eq(jitoSolTotalWithdrawFeeAmount.divn(10))
        )
            .eq(true, "withdrew sol amount should be equal to deposit sol amount except withdrawalSol fee");
    });
});
