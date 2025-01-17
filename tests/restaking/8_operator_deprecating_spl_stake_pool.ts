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

    step("withdraw SOL or stake", async function () {
        await restaking.sleep(1); // ...block hash not found?

        // @ts-ignore
        const jitoSolSupportedTokenAccount = restaking.knownAddress.fragSOLFundReserveSupportedTokenAccount("jitoSOL");
        const mSolSupportedTokenAccount = restaking.knownAddress.fragSOLFundReserveSupportedTokenAccount('mSOL');
        // @ts-ignore
        const bbSolSupportedTokenAccount = restaking.knownAddress.fragSOLFundReserveSupportedTokenAccount("bbSOL");
        const [
            fragSOLFundReserveAccountBalance0,
            jitoSolSupportedTokenBalance0,
            mSolSupportedTokenBalance0,
            bbSolSupportedTokenBalance0,
        ] = await Promise.all([
            restaking.getFragSOLFundReserveAccountBalance(),
            restaking.connection.getTokenAccountBalance(jitoSolSupportedTokenAccount, "confirmed")
                .then(v => new BN(v.value.amount)),
            restaking.connection.getTokenAccountBalance(mSolSupportedTokenAccount, "confirmed")
                .then(v => new BN(v.value.amount)),
            restaking.connection.getTokenAccountBalance(bbSolSupportedTokenAccount, "confirmed")
                .then(v => new BN(v.value.amount)),
        ]);
        logger.debug(`before withdraw-sol, fragSOLFundReserveAccountBalance=${fragSOLFundReserveAccountBalance0}`);
        logger.debug(`before withdraw-sol, jitoSolSupportedTokenBalance=${jitoSolSupportedTokenBalance0}`);
        logger.debug(`before withdraw-sol, mSolSupportedTokenBalance=${mSolSupportedTokenBalance0}`);
        logger.debug(`before withdraw-sol, bbSolSupportedTokenBalance=${bbSolSupportedTokenBalance0}`);

        await restaking.runOperatorFundCommands({
            command: {
                unstakeLst: {
                    0: {
                        state: {
                            prepare: {
                                items: [
                                    {
                                        tokenMint: restaking.supportedTokenMetadata["jitoSOL"].mint,
                                        allocatedTokenAmount: jitoSolSupportedTokenBalance0,
                                    },
                                    {
                                        tokenMint: restaking.supportedTokenMetadata["mSOL"].mint,
                                        allocatedTokenAmount: mSolSupportedTokenBalance0.divn(2), // not enough balance in pool reserve pda
                                    },
                                    {
                                        tokenMint: restaking.supportedTokenMetadata["bbSOL"].mint,
                                        allocatedTokenAmount: bbSolSupportedTokenBalance0,
                                    },
                                ],
                            },
                        },
                    }
                },
            },
            requiredAccounts: [],
        });

        const [
            fragSOLFundReserveAccountBalance1,
            jitoSolSupportedTokenBalance1,
            // jitoSolStakePoolInfo,
            mSolSupportedTokenBalance1,
            bbSolSupportedTokenBalance1,
            // bbSolStakePoolInfo,
        ] = await Promise.all([
            restaking.getFragSOLFundReserveAccountBalance(),
            restaking.connection.getTokenAccountBalance(jitoSolSupportedTokenAccount, "confirmed")
                .then(v => new BN(v.value.amount)),
            // restaking.getSplStakePoolInfo(new web3.PublicKey(restaking.getConstant("mainnetJitosolStakePoolAddress"))),
            restaking.connection.getTokenAccountBalance(mSolSupportedTokenAccount, "confirmed")
                .then(v => new BN(v.value.amount)),
            restaking.connection.getTokenAccountBalance(bbSolSupportedTokenAccount, "confirmed")
                .then(v => new BN(v.value.amount)),
            // restaking.getSplStakePoolInfo(new web3.PublicKey(restaking.getConstant('mainnetBbsolStakePoolAddress'))),
        ]);

        logger.debug(`after withdraw-sol, fragSOLFundReserveAccountBalance=${fragSOLFundReserveAccountBalance1}`);
        logger.debug(`after withdraw-sol, jitoSolSupportedTokenBalance=${jitoSolSupportedTokenBalance1}`);
        logger.debug(`after withdraw-sol, mSolSupportedTokenBalance=${mSolSupportedTokenBalance1}`);
        logger.debug(`after withdraw-sol, bbSolSupportedTokenBalance=${bbSolSupportedTokenBalance1}`);

        // const actualReserveDiff = new BN(fragSOLFundReserveAccountBalance1).sub(new BN(fragSOLFundReserveAccountBalance0));
        // const expectedReserveDiff = depositSolAmount.sub(jitoSolWithdrawFeeAmount).sub(bbSolWithdrawFeeAmount);
        // expect(actualReserveDiff.sub(expectedReserveDiff).abs().lte(new BN(1))) // 1 lamport diff?
        //     .eq(true, "withdrew sol amount should be equal to deposit sol amount except withdrawalSol fee");
    });

    step("claim sol and then will wait 2 epoch", async function () {
        await restaking.runOperatorFundCommands({
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

    step("claim sol after 2 epoch wait", async () => {
        const slotsPerEpoch = await restaking.connection.getEpochSchedule().then(e => e.slotsPerEpoch);
        await restaking.sleepUntil(3 * slotsPerEpoch);

        await restaking.runOperatorFundCommands({
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
    })
});
