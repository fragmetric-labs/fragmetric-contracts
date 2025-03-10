import {BN, web3} from '@coral-xyz/anchor';
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";
import { RestakingPlayground } from '../../tools/restaking/playground';

describe("operator_denormalize", async () => {
    const restaking = await restakingPlayground as RestakingPlayground;

    step("normalize", async function () {
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

    step("denormalize", async function() {
        await restaking.runOperatorFundCommands({
                command: {
                    denormalizeNt: {
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
});
