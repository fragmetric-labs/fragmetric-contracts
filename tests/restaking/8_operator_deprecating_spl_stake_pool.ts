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

    step("stake SOL to stake pools", async function () {
        await restaking.runUserDepositSOL(restaking.wallet, depositSolAmount);
        await restaking.runOperatorFundCommands({
            // @ts-ignore
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

        // @ts-ignore
        const jitoSolSupportedTokenAccount = restaking.knownAddress.fragSOLSupportedTokenAccount("jitoSOL");
        const mSOLSupportedTokenAccount = restaking.knownAddress.fragSOLSupportedTokenAccount('mSOL');
        // @ts-ignore
        const bbSOLSupportedTokenAccount = restaking.knownAddress.fragSOLSupportedTokenAccount("bbSOL");
        const [
            fragSOLFundReserveAccountBalance0,
            jitoSOLSupportedTokenBalance0,
            mSOLSupportedTokenBalance0,
            bbSOLSupportedTokenBalance0,
        ] = await Promise.all([
            restaking.getFragSOLFundReserveAccountBalance(),
            restaking.connection.getTokenAccountBalance(jitoSolSupportedTokenAccount, "confirmed")
                .then(balance => new BN(balance.value.amount)),
            restaking.connection.getTokenAccountBalance(mSOLSupportedTokenAccount, "confirmed")
                .then(balance => new BN(balance.value.amount)),
            restaking.connection.getTokenAccountBalance(bbSOLSupportedTokenAccount, "confirmed")
                .then(balance => new BN(balance.value.amount)),
        ]);
        logger.debug(`before withdraw-sol, fragSOLFundReserveAccountBalance=${fragSOLFundReserveAccountBalance0}`);
        logger.debug(`before withdraw-sol, jitoSOLSupportedTokenBalance=${jitoSOLSupportedTokenBalance0}`);
        logger.debug(`before withdraw-sol, mSOLSupportedTokenBalance=${mSOLSupportedTokenBalance0}`);
        logger.debug(`before withdraw-sol, bbSOLSupportedTokenBalance=${bbSOLSupportedTokenBalance0}`);

        await restaking.runOperatorFundCommands({
            // @ts-ignore
            command: {
                unstakeLst: {
                    0: {
                        items: [
                            {
                                tokenMint: restaking.supportedTokenMetadata["jitoSOL"].mint,
                                allocatedTokenAmount: jitoSOLSupportedTokenBalance0,
                            },
                            {
                                tokenMint: restaking.supportedTokenMetadata["mSOL"].mint,
                                allocatedTokenAmount: mSOLSupportedTokenBalance0,
                            },
                            {
                                tokenMint: restaking.supportedTokenMetadata["bbSOL"].mint,
                                allocatedTokenAmount: bbSOLSupportedTokenBalance0,
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

        // const [
        //     fragSOLFundReserveAccountBalance1,
        //     jitoSOLSupportedTokenBalance1,
        //     jitoSOLStakePoolInfo,
        //     bbSOLSupportedTokenBalance1,
        //     bbSOLStakePoolInfo,
        // ] = await Promise.all([
        //     restaking.getFragSOLFundReserveAccountBalance(),
        //     restaking.connection.getTokenAccountBalance(jitoSolSupportedTokenAccount, "confirmed")
        //         .then(balance => new BN(balance.value.amount)),
        //     restaking.getSplStakePoolInfo(new web3.PublicKey(restaking.getConstant("mainnetJitosolStakePoolAddress"))),
        //     restaking.connection.getTokenAccountBalance(bbSOLSupportedTokenAccount, "confirmed")
        //         .then(balance => new BN(balance.value.amount)),
        //     restaking.getSplStakePoolInfo(new web3.PublicKey(restaking.getConstant('mainnetBbsolStakePoolAddress'))),
        // ]);

        // const jitoSOLWithdrawFeeAmount = depositSolAmount.mul(jitoSOLStakePoolInfo.solWithdrawalFee.numerator).div(jitoSOLStakePoolInfo.solWithdrawalFee.denominator);

        // const bbSolWithdrawFeeAmount = depositSolAmount.mul(bbSOLStakePoolInfo.solWithdrawalFee.numerator).div(bbSOLStakePoolInfo.solWithdrawalFee.denominator);

        // logger.debug(`after withdraw-sol from jitoSOL stake pool: jitoSolSupportedTokenBalance=${jitoSOLSupportedTokenBalance1}, jitoSolWithdrawFeeAmount=${jitoSOLWithdrawFeeAmount}, fragSOLFundReserveAccountBalance=${fragSOLFundReserveAccountBalance1}`);
        // logger.debug(`after withdraw-sol from bbSOL stake pool: bbSolSupportedTokenBalance=${bbSOLSupportedTokenBalance1}, bbSolWithdrawFeeAmount=${bbSolWithdrawFeeAmount} fragSOLFundReserveAccountBalance=${fragSOLFundReserveAccountBalance1}`);

        // const actualReserveDiff = new BN(fragSOLFundReserveAccountBalance1).sub(new BN(fragSOLFundReserveAccountBalance0));
        // const expectedReserveDiff = depositSolAmount.sub(jitoSOLWithdrawFeeAmount).sub(bbSolWithdrawFeeAmount);
        // expect(actualReserveDiff.sub(expectedReserveDiff).abs().lte(new BN(1))) // 1 lamport diff?
        //     .eq(true, "withdrew sol amount should be equal to deposit sol amount except withdrawalSol fee");
    });

    step("claim sol", async function () {
        // console.log(`fundStakeAccounts:`, restaking.knownAddress.fundStakeAccounts);
        await restaking.runOperatorFundCommands({
            // @ts-ignore
            command: {
                claimUnstakedSol: {
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
});
