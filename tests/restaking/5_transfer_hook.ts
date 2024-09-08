import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import {RestakingPlayground} from "../../tools/restaking/playground";
import {BN} from "@coral-xyz/anchor";
import {expect} from "chai";
import {step} from "mocha-steps";

describe("transfer_hook", async function() {
    const playground = await RestakingPlayground.local(anchor.AnchorProvider.env());
    const user7 = playground.keychain.getKeypair('MOCK_USER7');
    const user8 = playground.keychain.getKeypair('MOCK_USER8');

    step("try airdrop SOL to mock accounts", async function () {
        await Promise.all([
            playground.tryAirdrop(user7.publicKey, 100),
            playground.tryAirdrop(user8.publicKey, 100),
        ]);

        await playground.sleep(1); // ...block hash not found?
    });

    const amountDepositedEach = new BN((10 ** 9) * 10);
    step("user7 deposit SOL to mint fragSOL and create accounts", async function() {
        await playground.runUserDepositSOL(user7, amountDepositedEach, null);
    });

    step("transfer fails from client-side SDK when dest PDA is not created yet", async function() {
        // ref: node_modules/@solana/spl-token/lib/cjs/extensions/transferHook/seeds.js
        await expect(playground.runTransfer(user7, user8.publicKey, amountDepositedEach)).rejectedWith(spl.TokenTransferHookAccountDataNotFound);
    });

    step("user8 deposit SOL to mint fragSOL and create accounts", async function() {
        await playground.runUserDepositSOL(user8, amountDepositedEach, null);
    });

    step("transfer blocked from onchain-side for now", async function() {
        await expect(playground.runTransfer(user7, user8.publicKey, amountDepositedEach)).rejectedWith('TokenNotTransferableError');
    });
});
