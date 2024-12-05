import {BN, web3} from "@coral-xyz/anchor";
import {expect} from "chai";
import {step} from "mocha-steps";
// @ts-ignore
import * as spl from "@solana/spl-token-3.x";
import {restakingPlayground} from "../restaking";
import {getLogger} from '../../tools/lib';

const {logger, LOG_PAD_SMALL, LOG_PAD_LARGE} = getLogger("restaking");

describe("operator_restake_with_normalize", async () => {
    const restaking = await restakingPlayground;

    step("restake LSTs to jito vault through normalizing", async function () {
        const restakeSolAmount = new BN(5 * web3.LAMPORTS_PER_SOL);
        await restaking.runOperatorRun({
                command: {
                    restakeVst: {
                        0: {
                            items: [
                                {
                                    vaultAddress: restaking.knownAddress.fragSOLJitoVaultAccount,
                                    solAmount: restakeSolAmount,
                                },
                            ],
                            state: {
                                init: {},
                            },
                            operationReservedRestakeToken: null
                        }
                    },
                },
                requiredAccounts: [],

            },
            restaking.keychain.getKeypair('FUND_MANAGER'),
            100,
            300_000,
        );
    });

    step("request VRT from jito restaking pool", async function () {
        const unrestakeSolAmount = new BN(4 * web3.LAMPORTS_PER_SOL);

        await restaking.runOperatorRun({
                command: {
                    unrestakeVrt: {
                        0: {
                            items: [
                                {
                                    vaultAddress: restaking.knownAddress.fragSOLJitoVaultAccount,
                                    solAmount: unrestakeSolAmount,
                                },
                            ],
                            state: {
                                init: {},
                            },
                        }
                    },
                },
                requiredAccounts: [],

            },
            restaking.keychain.getKeypair('FUND_MANAGER'),
            100,
            3_000_000,
        );
    });
    step("request VRT from jito restaking pool2", async function () {
        const unrestakeSolAmount = new BN(1 * web3.LAMPORTS_PER_SOL);

        await restaking.runOperatorRun({
                command: {
                    unrestakeVrt: {
                        0: {
                            items: [
                                {
                                    vaultAddress: restaking.knownAddress.fragSOLJitoVaultAccount,
                                    solAmount: unrestakeSolAmount,
                                },
                            ],
                            state: {
                                init: {},
                            },
                        }
                    },
                },
                requiredAccounts: [],

            },
            restaking.keychain.getKeypair('FUND_MANAGER'),
            100,
            3_000_000,
        );
    });

    step("claim VRT from jito restaking pool", async function () {
        let currentSlot = await restaking.connection.getSlot();
        let afterSlot: number;
        console.log("waiting for cooling down")
        do {
            await new Promise(r => setTimeout(r, 20 * 1000))
            afterSlot = await restaking.connection.getSlot();
        } while (Math.floor((afterSlot - currentSlot) / 32) < 2);
        console.log("start claim vrt")

        const unrestakeSolAmount = new BN(4 * web3.LAMPORTS_PER_SOL);

        await restaking.runOperatorRun({
                command: {
                    claimUnrestakedVst: {
                        0: {
                            items: [
                                {
                                    vaultAddress: restaking.knownAddress.fragSOLJitoVaultAccount,
                                },
                            ],
                            state: {
                                init: {},
                            },
                        }
                    },
                },
                requiredAccounts: [],

            },
            restaking.keychain.getKeypair('FUND_MANAGER'),
            100,
            3_000_000,
        );

        let fundAccount = await restaking.getFragSOLFundAccount()
        if (fundAccount.operation.nextCommand == null) {

        }

    });
});
