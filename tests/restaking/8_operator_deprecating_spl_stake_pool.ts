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
        await restaking.runUserDepositSOL(restaking.wallet, depositSolAmount);
        await restaking.runOperatorFundCommands({
            command: {
                stakeSol: {
                    0: {
                        state: {
                            new: {},
                        },
                    }
                },
            },
            requiredAccounts: [],
        });
    });

    step("withdraw SOL from bbSOL stake pool", async function () {
        await restaking.sleep(1); // ...block hash not found?

        const bbSolSupportedTokenAccount = restaking.knownAddress.fragSOLSupportedTokenAccount("bbSOL");
        const [
            fragSOLFundReserveAccountBalance0,
            bbSolSupportedTokenBalance0_,
        ] = await Promise.all([
            restaking.getFragSOLFundReserveAccountBalance(),
            restaking.connection.getTokenAccountBalance(bbSolSupportedTokenAccount, "confirmed"),
        ]);
        const bbSolSupportedTokenBalance0 = new BN(bbSolSupportedTokenBalance0_.value.amount)
        logger.debug(`before withdraw-sol from bbSOL stake pool: bbSolSupportedTokenBalance=${bbSolSupportedTokenBalance0}, fragSOLFundReserveAccountBalance=${fragSOLFundReserveAccountBalance0}`);

        await restaking.runOperatorFundCommands({
            command: {
                unstakeLst: {
                    0: {
                        items: [
                            {
                                mint: restaking.supportedTokenMetadata.bbSOL.mint,
                                tokenAmount: bbSolSupportedTokenBalance0,
                            },
                        ],
                        state: {
                            init: {},
                        },
                    }
                },
            },
            requiredAccounts: [],
        });

        const [
            fragSOLFundReserveAccountBalance1,
            bbSolSupportedTokenBalance1_,
            bbSolStakePoolInfo,
        ] = await Promise.all([
            restaking.getFragSOLFundReserveAccountBalance(),
            restaking.connection.getTokenAccountBalance(bbSolSupportedTokenAccount, "confirmed"),
            restaking.getSplStakePoolInfo(new web3.PublicKey(restaking.getConstant('mainnetBbsolStakePoolAddress'))),
        ]);
        const bbSolSupportedTokenBalance1 = new BN(bbSolSupportedTokenBalance1_.value.amount);
        const WithdrawFeeAmount = depositSolAmount.mul(bbSolStakePoolInfo.solWithdrawalFee.numerator).div(bbSolStakePoolInfo.solWithdrawalFee.denominator);

        logger.debug(`after withdraw-sol from bbSOL stake pool: bbSolSupportedTokenBalance=${bbSolSupportedTokenBalance1}, WithdrawFeeAmount=${WithdrawFeeAmount} fragSOLFundReserveAccountBalance=${fragSOLFundReserveAccountBalance1}`);

        const actualReserveDiff = new BN(fragSOLFundReserveAccountBalance1).sub(new BN(fragSOLFundReserveAccountBalance0));
        const expectedReserveDiff = depositSolAmount.sub(WithdrawFeeAmount);
        // expect(actualReserveDiff.sub(expectedReserveDiff).abs().lte(new BN(1))) // 1 lamport diff?
        //     .eq(true, "withdrew sol amount should be equal to deposit sol amount except withdrawalSol fee");
    });

    step("claim sol", async function () {
        // console.log(`fundStakeAccounts:`, restaking.knownAddress.fundStakeAccounts);
        await restaking.runOperatorFundCommands({
            command: {
                claimUnstakedSol: {
                    0: {
                        items: [
                            {
                                mint: restaking.supportedTokenMetadata.bbSOL.mint,
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
        });
    });
});
