import {BN, web3} from "@coral-xyz/anchor";
import {expect} from "chai";
import {step} from "mocha-steps";
// @ts-ignore
import * as spl from "@solana/spl-token-3.x";
import {restakingPlayground} from "../restaking";
import {getLogger} from '../../tools/lib';

const {logger, LOG_PAD_SMALL, LOG_PAD_LARGE} = getLogger("restaking");

describe("operator_spl_stake_pool", async () => {
    const restaking = await restakingPlayground;

    const depositSolAmount = new BN(100_000 * web3.LAMPORTS_PER_SOL);
    // const depositSolAmount = new BN(500 * web3.LAMPORTS_PER_SOL);

    step("stake SOL to jito stake pool", async function () {
        await restaking.runUserDepositSOL(restaking.wallet, depositSolAmount, null);
        await restaking.runOperatorRun({
            command: {
                stakeSol: {
                    0: {
                        items: [
                            {
                                mint: restaking.supportedTokenMetadata.jitoSOL.mint,
                                solAmount: depositSolAmount,
                            },
                        ],
                        state: {
                            init: {},
                        },
                    }
                },
            },
            requiredAccounts: [],
        }, undefined, 3);
    });

    step("withdraw SOL from jito stake pool", async function () {
        await restaking.sleep(1); // ...block hash not found?

        const jitoSolSupportedTokenAccount = restaking.knownAddress.fragSOLSupportedTokenAccount("jitoSOL");
        const [
            fragSOLFundReserveAccountBalance0,
            jitoSolSupportedTokenBalance0_,
        ] = await Promise.all([
            restaking.getFragSOLFundReserveAccountBalance(),
            restaking.connection.getTokenAccountBalance(jitoSolSupportedTokenAccount, "confirmed"),
        ]);
        const jitoSolSupportedTokenBalance0 = new BN(jitoSolSupportedTokenBalance0_.value.amount)
        logger.debug(`before withdraw-sol from jito stake pool: jitoSolSupportedTokenBalance=${jitoSolSupportedTokenBalance0}, fragSOLFundReserveAccountBalance=${fragSOLFundReserveAccountBalance0}`);

        await restaking.runOperatorRun({
            command: {
                unstakeLst: {
                    0: {
                        items: [
                            {
                                mint: restaking.supportedTokenMetadata.jitoSOL.mint,
                                tokenAmount: jitoSolSupportedTokenBalance0,
                            },
                        ],
                        state: {
                            init: {},
                        },
                    }
                },
            },
            requiredAccounts: [],
        }, undefined, 3, 800_000);
        
        const [
            fragSOLFundReserveAccountBalance1,
            jitoSolSupportedTokenBalance1_,
            jitoStakePoolInfo,
        ] = await Promise.all([
            restaking.getFragSOLFundReserveAccountBalance(),
            restaking.connection.getTokenAccountBalance(jitoSolSupportedTokenAccount, "confirmed"),
            restaking.getSplStakePoolInfo(new web3.PublicKey(restaking.getConstant('mainnetJitosolStakePoolAddress'))),
        ]);
        const jitoSolSupportedTokenBalance1 = new BN(jitoSolSupportedTokenBalance1_.value.amount);
        const WithdrawFeeAmount = depositSolAmount.mul(jitoStakePoolInfo.solWithdrawalFee.numerator).div(jitoStakePoolInfo.solWithdrawalFee.denominator);

        logger.debug(`after withdraw-sol from jito stake pool: jitoSolSupportedTokenBalance=${jitoSolSupportedTokenBalance1}, WithdrawFeeAmount=${WithdrawFeeAmount} fragSOLFundReserveAccountBalance=${fragSOLFundReserveAccountBalance1}`);

        const actualReserveDiff = new BN(fragSOLFundReserveAccountBalance1).sub(new BN(fragSOLFundReserveAccountBalance0));
        const expectedReserveDiff = depositSolAmount.sub(WithdrawFeeAmount);
        // expect(actualReserveDiff.sub(expectedReserveDiff).abs().lte(new BN(1))) // 1 lamport diff?
        //     .eq(true, "withdrew sol amount should be equal to deposit sol amount except withdrawalSol fee");
    });

    step("claim sol", async function () {
        console.log(`fundStakeAccounts:`, restaking.knownAddress.fundStakeAccounts);
        await restaking.runOperatorRun({
            command: {
                claimUnstakedSol: {
                    0: {
                        items: [
                            {
                                mint: restaking.supportedTokenMetadata.jitoSOL.mint,
                                fundStakeAccounts: restaking.knownAddress.fundStakeAccounts,
                            },
                        ],
                        state: {
                            init: {},
                        },
                    }
                },
            },
            requiredAccounts: [],
        }, undefined, 3, 800_000);
    });
});
