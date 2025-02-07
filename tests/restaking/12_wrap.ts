// @ts-ignore
import * as spl from '@solana/spl-token-3.x';
import {BN, IdlAccounts, web3} from '@coral-xyz/anchor';
import {expect} from 'chai';
import {step} from 'mocha-steps';
import {restakingPlayground} from '../restaking';
import { Restaking } from '../../target/types/restaking';
import { getLogger } from '../../tools/lib';
import { RestakingPlayground } from '../../tools/restaking/playground';

const {logger} = getLogger('reward');

describe("wrap", async function () {
    const restaking = await restakingPlayground as RestakingPlayground;
    const userA = restaking.keychain.getKeypair('MOCK_USER11');
    const userB = restaking.keychain.getKeypair('MOCK_USER12');

    step("try airdrop SOL to mock accounts", async function () {
        await Promise.all([
            restaking.tryAirdrop(userA.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(100)),
            restaking.tryAirdrop(userB.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(100)),
        ]);
    });

    const amountDepositedEach = new BN((10 ** restaking.fragSOLDecimals) * 10);
    step("userA deposit SOL (with metadata) to mint fragSOL and create accounts", async function () {
        await restaking.runUserDepositSOL(userA, amountDepositedEach, null);
    });

    step("userA wraps exact amount of fragSOL and transfer to userB", async function () {
        // wrap 5 fragSOL, wrap 5 fragSOL then send 10 wFragSOL
        // check global & user A reward: token allocated amount decreased (10)
    })

    step("userB unwraps fragSOL but still reward not activated", async () => {
        // unwrap 10 wFragSOL
        // check global & user B reward: token allocated amount no change
    })

    step("userB wraps fragSOL and still reward not activated", async () => {
        // wrap 5 fragSOL
        // check global & user B reward: token allocated amount no change
    })

    step("userB create accounts", async () => {
        // deposit 10 SOL
        // check global & user B reward: token allocated amount increased (5)
        // check userB does not have 1.3 rate
    })

    step("userB wraps desired amount of fragSOL and transfer to userA", async () => {
        // wrap until 10 wFragSOL(5 fragSOL wrapped) then send 10 wFragSOL
    })
});
