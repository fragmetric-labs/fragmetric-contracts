import {BN} from '@coral-xyz/anchor';
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

    step("update fragSOL token metadata", async function () {
        await restaking.runAdminUpdateTokenMetadata();
    });

    step("create nSOL token mint", async function () {
        const { nSOLMint } = await restaking.runAdminInitializeNSOLTokenMint();
        expect(nSOLMint.address.toString()).eq(restaking.knownAddress.nSOLTokenMint.toString());
        expect(nSOLMint.mintAuthority.toString()).eq(restaking.keychain.getKeypair('ADMIN').publicKey.toString());
        expect(nSOLMint.freezeAuthority).null;
    })

    step("initialize fund accounts", async () => {
        const { fragSOLFundAccount } = await restaking.runAdminInitializeFundAccounts();

        expect(fragSOLFundAccount.dataVersion).gt(0);

        await restaking.runAdminUpdateFundAccounts();

        expect(fragSOLFundAccount.dataVersion).gt(1);
    })

    step("initialize normalized token pool", async () => {
        const { nSOLTokenPoolAccount } = await restaking.runAdminInitializeNormalizeTokenPool();

        expect(nSOLTokenPoolAccount.normalizedTokenMint.toString()).eq(restaking.knownAddress.nSOLTokenMint.toString());
    })

    step("initialize jito restaking protocol account", async () => {
        const { fragSOLJitoVRTAccount } = await restaking.runAdminInitializeJitoRestakingProtocolAccount();

        expect(fragSOLJitoVRTAccount.mint.toString()).eq(restaking.knownAddress.fragSOLJitoVRTMint.toString());
        expect(fragSOLJitoVRTAccount.owner.toString()).eq(restaking.knownAddress.fragSOLFund.toString());
    })

    step("initialize reward accounts", async () => {
        const { fragSOLRewardAccount } = await restaking.runAdminUpdateRewardAccounts();

        expect(fragSOLRewardAccount.dataVersion).eq(parseInt(restaking.getConstant('rewardAccountCurrentVersion')));
    })

    step("transfer token mint authority to PDA", async function () {
        const {
            fragSOLMint,
            fragSOLExtraAccountMetasAccount,
        } = await restaking.runAdminTransferMintAuthority();

        expect(fragSOLMint.mintAuthority.toString()).eq(restaking.knownAddress.fragSOLTokenMintAuthority.toString());
        expect(fragSOLExtraAccountMetasAccount.length).eq(6);
    });

    step("initialize fund and supported tokens configuration", async function () {
        const _ = await restaking.runFundManagerInitializeFundConfigurations();
        const res0 = await restaking.runFundManagerUpdateFundConfigurations();

        expect(res0.fragSOLFund.supportedTokens.length).eq(Object.values(restaking.supportedTokenMetadata).length);
        let i = 0;
        for (const v of Object.values(restaking.supportedTokenMetadata)) {
            const supported = res0.fragSOLFund.supportedTokens[i++];
            expect(supported.mint.toString()).eq(v.mint.toString());
            expect(supported.program.toString()).eq(v.program.toString());
            expect(supported.oneTokenAsSol.toNumber()).greaterThan(0);
            expect(supported.operationReservedAmount.toNumber()).eq(0);
        }
    });

    step("initialize normalized token pool supported tokens configuration", async function() {
        const { nSOLTokenPool } = await restaking.runFundManagerInitializeNormalizeTokenPoolConfigurations();

        expect(nSOLTokenPool.supportedTokens.length).eq(Object.values(restaking.supportedTokenMetadata).length);
        let i = 0;
        for (const v of Object.values(restaking.supportedTokenMetadata)) {
            const supported = nSOLTokenPool.supportedTokens[i++];
            expect(supported.mint.toString()).eq(v.mint.toString());
            expect(supported.program.toString()).eq(v.program.toString());
            expect(supported.lockedAmount.toNumber()).eq(0);
        }
    })

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
