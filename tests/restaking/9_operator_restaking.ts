import {step} from "mocha-steps";
// @ts-ignore
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
        await restaking.runOperatorFundCommands({
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
    });

    step("request VRT from jito restaking pool", async function () {
        await restaking.runOperatorFundCommands({
                command: {
                    unrestakeVrt: {
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
    });
});
