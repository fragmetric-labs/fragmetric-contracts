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

    step("normalize before restake", async function () {
        await restaking.runOperatorFundCommands({
                command: {
                    normalizeSt: {
                        0: {
                            state: {
                                new: {},
                            },
                        }
                    },
                },
                requiredAccounts: [],
            },
            restaking.keychain.getKeypair("FUND_MANAGER"),
        );
    });

    step("restake LSTs to jito vault through normalizing", async function () {
        let {event} = await restaking.runOperatorFundCommands({
                command: {
                    restakeVst: {
                        0: {
                            state: {
                                new: {},
                            },
                        }
                    },
                },
                requiredAccounts: [],

            },
            restaking.keychain.getKeypair('FUND_MANAGER'),
        );
        console.log(`restaking command event:`, event);
    });

    // step("request VRT from jito restaking pool", async function () {
    //     const unrestakeSolAmount = new BN(4 * web3.LAMPORTS_PER_SOL);
    //     await restaking.runOperatorFundCommands({
    //             command: {
    //                 unrestakeVrt: {
    //                     0: {
    //                         items: [
    //                             {
    //                                 vaultAddress: restaking.knownAddress.fragSOLJitoVaultAccount,
    //                                 solAmount: unrestakeSolAmount,
    //                             },
    //                         ],
    //                         state: {
    //                             init: {},
    //                         },
    //                     }
    //                 },
    //             },
    //             requiredAccounts: [],

    //         },
    //         restaking.keychain.getKeypair('FUND_MANAGER'),
    //     );
    // });
});
