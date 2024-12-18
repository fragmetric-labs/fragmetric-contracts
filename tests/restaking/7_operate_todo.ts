import {BN} from "@coral-xyz/anchor";
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";
import {getLogger} from '../../tools/lib';

const {logger, LOG_PAD_SMALL, LOG_PAD_LARGE} = getLogger("restaking");

module.exports = (i: number) => describe(`operate#TODO${i}`, async () => {
    const restaking = await restakingPlayground;

    step("fund operation for a single cycle", async function () {
        await restaking.runOperatorFundCommands();
    });
});
