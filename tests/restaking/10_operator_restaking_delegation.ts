import {BN, web3} from '@coral-xyz/anchor';
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";

describe("operator_restaking_delegation", async () => {
    const restaking = await restakingPlayground;

    // dev) run just once if there's no Jito restaking operator account file
    // step("initialize new operator", async function () {
    //     await restaking.runAdminInitializeJitoRestakingOperator();
    // });

    // dev) run just once if you want to set the Jito vault's vault_delegation_admin account to fund_account
    // you can call it on REPL
    // await restaking.runAdminSetSecondaryAdminForJitoVault();

    step("initialize operator_vault_ticket & vault_operator_delegation", async function() {
        const operator = new web3.PublicKey("2p4kQZTYL3jKHpkjTaFULvqcKNsF8LoeFGEHWYt2sJAV");

        const {operatorVaultTicket} = await restaking.runAdminInitializeOperatorVaultTicket(operator);
        await restaking.runAdminInitializeVaultOperatorDelegation(operator, operatorVaultTicket[0]);
    });

    step("initialize vault delegation at fund account", async function() {
        const operator = new web3.PublicKey("2p4kQZTYL3jKHpkjTaFULvqcKNsF8LoeFGEHWYt2sJAV");
        await restaking.runFundManagerInitializeFundJitoRestakingVaultDelegation(operator);
    });

    step("run command add_delegation", async function() {
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
});
