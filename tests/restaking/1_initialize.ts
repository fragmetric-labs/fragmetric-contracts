import * as anchor from "@coral-xyz/anchor";
import { BN } from '@coral-xyz/anchor';;
// @ts-ignore
import * as spl from "@solana/spl-token";
import { expect } from "chai";
import {RestakingPlayground} from "../../tools/restaking/playground";
import {step} from "mocha-steps";

describe("initialize", async () => {
    const playground = await RestakingPlayground.local(anchor.AnchorProvider.env());

    step("create fragSOL token mint with extensions", async function() {
        const res0 = await playground.runAdminInitializeTokenMint();
        expect(res0.fragSOLMint.address.toString()).eq(playground.knownAddress.fragSOLTokenMint.toString());
        expect(res0.fragSOLMint.mintAuthority.toString()).eq(playground.keychain.getKeypair('ADMIN').publicKey.toString()); // shall be transferred to a PDA below
        expect(res0.fragSOLMint.freezeAuthority).null;
    });

    step("mock supported token mints", async function() {
        const tokenMint_bSOL = await spl.getMint(
            playground.connection,
            playground.supportedTokenMetadata.bSOL.mint,
        );
        const tokenMint_mSOL = await spl.getMint(
            playground.connection,
            playground.supportedTokenMetadata.mSOL.mint,
        );
        const tokenMint_jitoSOL = await spl.getMint(
            playground.connection,
            playground.supportedTokenMetadata.jitoSOL.mint,
        );

        expect(tokenMint_bSOL.mintAuthority.toString()).eq(playground.keychain.getKeypair('MOCK_ALL_MINT_AUTHORITY').publicKey.toString());
        expect(tokenMint_mSOL.mintAuthority.toString()).eq(playground.keychain.getKeypair('MOCK_ALL_MINT_AUTHORITY').publicKey.toString());
        expect(tokenMint_jitoSOL.mintAuthority.toString()).eq(playground.keychain.getKeypair('MOCK_ALL_MINT_AUTHORITY').publicKey.toString());
    });

    step("initialize fund, reward accounts, token extensions and transfer token mint authority to PDA", async function() {
        const res0 = await playground.runAdminInitializeFundAndRewardAccountsAndMint();

        expect(res0.fragSOLFundAccount.dataVersion).gt(0);
        expect(res0.fragSOLRewardAccount.dataVersion).eq(parseInt(playground.getConstant('rewardAccountCurrentVersion')));
        expect(res0.fragSOLMint.mintAuthority.toString()).eq(playground.knownAddress.fragSOLTokenMintAuthority.toString());
        expect(res0.fragSOLExtraAccountMetasAccount.length).eq(6);
    });

    step("initialize fund and supported tokens configuration", async function() {
        const res0 = await playground.runFundManagerInitializeFundConfigurations();

        expect(res0.fragSOLFund.supportedTokens.length).eq(Object.values(playground.supportedTokenMetadata).length);
        let i = 0;
        for (const v of Object.values(playground.supportedTokenMetadata)) {
            const supported = res0.fragSOLFund.supportedTokens[i++];
            expect(supported.mint.toString()).eq(v.mint.toString());
            expect(supported.program.toString()).eq(v.program.toString());
            expect(supported.price.toNumber()).greaterThan(0);
            expect(supported.operationReservedAmount.toNumber()).eq(0);
        }
    });

    step("initialize reward pools and rewards", async function() {
        const res0 = await playground.runFundManagerInitializeRewardPools();

        expect(res0.fragSOLReward.dataVersion).eq(parseInt(playground.getConstant('rewardAccountCurrentVersion')));

        expect(res0.fragSOLReward.numRewards).eq(Object.values(playground.rewardsMetadata).length);
        let i = 0;
        for (const v of Object.values(playground.rewardsMetadata)) {
            const reward = res0.fragSOLReward.rewards1[i++];
            expect(playground.binToString(reward.name)).eq(v.name.toString());
            expect(playground.binToString(reward.description)).eq(v.description.toString());
        }

        expect(res0.fragSOLReward.numRewardPools).eq(Object.values(playground.rewardPoolsMetadata).length);
        i = 0;
        for (const v of Object.values(playground.rewardPoolsMetadata)) {
            const pool = res0.fragSOLReward.rewardPools1[i++];
            expect(playground.binToString(pool.name)).eq(v.name.toString());
        }
    });

    step("settle fPoint reward (zeroing)", async () => {
        await new Promise(resolve => setTimeout(resolve, 1000)); // wait for few slot elapsed
        const res0 = await playground.runFundManagerSettleReward({
            poolName: 'bonus',
            rewardName: 'fPoint',
            amount: new BN(0),
        });
        expect(res0.fragSOLReward.rewardPools1[res0.rewardPool.id].numRewardSettlements).eq(1);
        expect(res0.fragSOLReward.rewardPools1[res0.rewardPool.id].rewardSettlements1[0].rewardId).eq(res0.reward.id);
        expect(res0.fragSOLReward.rewardPools1[res0.rewardPool.id].rewardSettlements1[0].rewardPoolId).eq(res0.rewardPool.id);
        expect(res0.fragSOLReward.rewardPools1[res0.rewardPool.id].rewardSettlements1[0].numSettlementBlocks).eq(1);
        expect(res0.fragSOLReward.rewardPools1[res0.rewardPool.id].rewardSettlements1[0].settledAmount.toNumber()).eq(0);
    });
});
