import {BN, web3} from '@coral-xyz/anchor';
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";
import { RestakingPlayground } from '../../tools/restaking/playground';

describe("operator_restaking_delegation", async () => {
    const restaking = await restakingPlayground as RestakingPlayground;

    step("delegate", async function() {
        await restaking.runOperatorFundCommands({
                command: {
                    delegateVst: {
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

    step("undelegate", async function() {
        await restaking.runOperatorFundCommands({
                command: {
                    undelegateVst: {
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
