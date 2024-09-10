import {BN} from '@coral-xyz/anchor';
// @ts-ignore
import * as spl from "@solana/spl-token";
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";

describe("initialize", async () => {
    const restaking = await restakingPlayground;

    step("create fragSOL token mint with extensions", async function () {
        const res0 = await restaking.runAdminInitializeTokenMint();
        expect(res0.fragSOLMint.address.toString()).eq(restaking.knownAddress.fragSOLTokenMint.toString());
        expect(res0.fragSOLMint.mintAuthority.toString()).eq(restaking.keychain.getKeypair('ADMIN').publicKey.toString()); // shall be transferred to a PDA below
        expect(res0.fragSOLMint.freezeAuthority).null;
    });

    step("mock supported token mints", async function () {
        const tokenMint_bSOL = await spl.getMint(
            restaking.connection,
            restaking.supportedTokenMetadata.bSOL.mint,
        );
        const tokenMint_mSOL = await spl.getMint(
            restaking.connection,
            restaking.supportedTokenMetadata.mSOL.mint,
        );
        const tokenMint_jitoSOL = await spl.getMint(
            restaking.connection,
            restaking.supportedTokenMetadata.jitoSOL.mint,
        );

        expect(tokenMint_bSOL.mintAuthority.toString()).eq(restaking.keychain.getKeypair('MOCK_ALL_MINT_AUTHORITY').publicKey.toString());
        expect(tokenMint_mSOL.mintAuthority.toString()).eq(restaking.keychain.getKeypair('MOCK_ALL_MINT_AUTHORITY').publicKey.toString());
        expect(tokenMint_jitoSOL.mintAuthority.toString()).eq(restaking.keychain.getKeypair('MOCK_ALL_MINT_AUTHORITY').publicKey.toString());
    });

    step("initialize fund accounts", async () => {
        const { fragSOLFundAccount } = await restaking.runAdminInitializeFundAccounts();

        expect(fragSOLFundAccount.dataVersion).gt(0);
    })

    step("initialize reward accounts", async () => {
        const { fragSOLRewardAccount } = await restaking.runAdminUpdateRewardAccounts();

        expect(fragSOLRewardAccount.dataVersion).eq(parseInt(restaking.getConstant('rewardAccountCurrentVersion')));
    })

    step("transfer token mint authority to PDA", async function () {
        const {
            fragSOLMint,
            fragSOLExtraAccountMetasAccount,
        } = await restaking.runAdminInitializeMint();

        expect(fragSOLMint.mintAuthority.toString()).eq(restaking.knownAddress.fragSOLTokenMintAuthority.toString());
        expect(fragSOLExtraAccountMetasAccount.length).eq(6);
    });

    step("initialize fund and supported tokens configuration", async function () {
        const res0 = await restaking.runFundManagerInitializeFundConfigurations();

        expect(res0.fragSOLFund.supportedTokens.length).eq(Object.values(restaking.supportedTokenMetadata).length);
        let i = 0;
        for (const v of Object.values(restaking.supportedTokenMetadata)) {
            const supported = res0.fragSOLFund.supportedTokens[i++];
            expect(supported.mint.toString()).eq(v.mint.toString());
            expect(supported.program.toString()).eq(v.program.toString());
            expect(supported.price.toNumber()).greaterThan(0);
            expect(supported.operationReservedAmount.toNumber()).eq(0);
        }
    });

    step("initialize reward pools and rewards", async function () {
        const res0 = await restaking.runFundManagerInitializeRewardPools();

        expect(res0.fragSOLReward.dataVersion).eq(parseInt(restaking.getConstant('rewardAccountCurrentVersion')));

        expect(res0.fragSOLReward.numRewards).eq(Object.values(restaking.rewardsMetadata).length);
        let i = 0;
        for (const v of Object.values(restaking.rewardsMetadata)) {
            const reward = res0.fragSOLReward.rewards1[i++];
            expect(restaking.binToString(reward.name)).eq(v.name.toString());
            expect(restaking.binToString(reward.description)).eq(v.description.toString());
        }

        expect(res0.fragSOLReward.numRewardPools).eq(Object.values(restaking.rewardPoolsMetadata).length);
        i = 0;
        for (const v of Object.values(restaking.rewardPoolsMetadata)) {
            const pool = res0.fragSOLReward.rewardPools1[i++];
            expect(restaking.binToString(pool.name)).eq(v.name.toString());
        }
    });

    step("settle fPoint reward (zeroing)", async () => {
        await new Promise(resolve => setTimeout(resolve, 1000)); // wait for few slot elapsed
        const res0 = await restaking.runFundManagerSettleReward({
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
