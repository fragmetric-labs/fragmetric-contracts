// @ts-ignore
import * as spl from '@solana/spl-token-3.x';
import {BN, web3} from '@coral-xyz/anchor';
import {expect} from 'chai';
import {step} from 'mocha-steps';
import {restakingPlayground} from '../restaking';

describe("transfer_hook", async function () {
    const restaking = await restakingPlayground;
    const user7 = restaking.keychain.getKeypair('MOCK_USER7');
    const user8 = restaking.keychain.getKeypair('MOCK_USER8');

    step("try airdrop SOL to mock accounts", async function () {
        await Promise.all([
            restaking.tryAirdrop(user7.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(100)),
            restaking.tryAirdrop(user8.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(100)),
        ]);

        await restaking.sleep(1); // ...block hash not found?
    });

    const amountDepositedEach = new BN((10 ** restaking.fragSOLDecimals) * 10);
    step("user7 deposit SOL to mint fragSOL and create accounts", async function () {
        await restaking.runUserDepositSOL(user7, amountDepositedEach, null);
    });

    step("transfer fails from client-side SDK when dest PDA is not created yet", async function () {
        // ref: node_modules/@solana/spl-token/lib/cjs/extensions/transferHook/seeds.js
        await expect(restaking.runTransfer(user7, user8.publicKey, amountDepositedEach)).rejectedWith(spl.TokenTransferHookAccountDataNotFound);
    });

    step("user8 deposit SOL to mint fragSOL and create accounts", async function () {
        await restaking.runUserDepositSOL(user8, amountDepositedEach, null);
    });

    step("transfer blocked from onchain-side for now", async function () {
        await expect(restaking.runTransfer(user7, user8.publicKey, amountDepositedEach)).rejectedWith('TokenNotTransferableError');
    });
});
