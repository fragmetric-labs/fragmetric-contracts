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

    const depositSOLAmount = new BN(100_000 * web3.LAMPORTS_PER_SOL);
    // const depositSolAmount = new BN(500 * web3.LAMPORTS_PER_SOL);

    step("stake SOL to stake pools", async function () {
        await restaking.runUserDepositSOL(restaking.wallet, depositSOLAmount);
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

        // @ts-ignore
        const jitoSOLSupportedTokenAccount = restaking.knownAddress.fragSOLSupportedTokenAccount("jitoSOL");
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
            restaking.connection.getTokenAccountBalance(jitoSOLSupportedTokenAccount, "confirmed")
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

        const [
            fragSOLFundReserveAccountBalance1,
            jitoSOLSupportedTokenBalance1,
            mSOLSupportedTokenBalance1,
            bbSOLSupportedTokenBalance1,
            jitoSOLStakePoolInfo,
            bbSOLStakePoolInfo,
        ] = await Promise.all([
            restaking.getFragSOLFundReserveAccountBalance(),
            restaking.connection.getTokenAccountBalance(jitoSOLSupportedTokenAccount, "confirmed")
                .then(balance => new BN(balance.value.amount)),
            restaking.connection.getTokenAccountBalance(mSOLSupportedTokenAccount, "confirmed")
                .then(balance => new BN(balance.value.amount)),
            restaking.connection.getTokenAccountBalance(bbSOLSupportedTokenAccount, "confirmed")
                .then(balance => new BN(balance.value.amount)),
            restaking.getSplStakePoolInfo(restaking.getConstantAsPublicKey('mainnetJitosolStakePoolAddress')),
            restaking.getSplStakePoolInfo(restaking.getConstantAsPublicKey('mainnetBbsolStakePoolAddress')),
        ]);

        const jitoSOLWithdrawalFeeAmount = jitoSOLSupportedTokenBalance0.mul(jitoSOLStakePoolInfo.solWithdrawalFee.numerator).div(jitoSOLStakePoolInfo.solWithdrawalFee.denominator);
        const bbSOLWithdrawalFeeAmount = bbSOLSupportedTokenBalance0.mul(bbSOLStakePoolInfo.solWithdrawalFee.numerator).div(bbSOLStakePoolInfo.solWithdrawalFee.denominator);

        logger.debug(`after withdraw-sol, fragSOLFundReserveAccountBalance=${fragSOLFundReserveAccountBalance1}`);
        logger.debug(`after withdraw-sol, jitoSolSupportedTokenBalance=${jitoSOLSupportedTokenBalance1}, jitoSolWithdrawFeeAmount=${jitoSOLWithdrawalFeeAmount}`);
        logger.debug(`after withdraw-sol, mSOLSupportedTokenBalance=${mSOLSupportedTokenBalance1}`);
        logger.debug(`after withdraw-sol, bbSolSupportedTokenBalance=${bbSOLSupportedTokenBalance1}, bbSolWithdrawFeeAmount=${bbSOLWithdrawalFeeAmount}`);

        // const actualReserveDiff = new BN(fragSOLFundReserveAccountBalance1).sub(new BN(fragSOLFundReserveAccountBalance0));
        // const expectedReserveDiff = depositSOLAmount.sub(jitoSOLWithdrawalFeeAmount).sub(bbSOLWithdrawalFeeAmount);
        // expect(actualReserveDiff.sub(expectedReserveDiff).abs().lte(new BN(1))) // 1 lamport diff?
        //     .eq(true, "withdrew sol amount should be equal to deposit sol amount except withdrawalSol fee");
    });


    step("claim sol after 2 epoch shift (epoch = 32 slot)", async function () {
        await restaking.sleepUntil(96);
        // console.log(`fundStakeAccounts:`, restaking.knownAddress.fundStakeAccounts);
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
});
