import {BN, web3} from '@coral-xyz/anchor';
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";


describe("initialize", async () => {
    const restaking = await restakingPlayground;

    step("try airdrop SOL to authorized wallets", async function () {
        await Promise.all([
            restaking.tryAirdrop(restaking.keychain.getPublicKey('ADMIN'), 100),
            restaking.tryAirdrop(restaking.keychain.getPublicKey('FUND_MANAGER'), 100),
        ]);

        await restaking.sleep(1); // ...block hash not found?
    });

    step("create known address lookup table", async function () {
        await restaking.getOrCreateKnownAddressLookupTable();
    });

    step("create fragSOL token mint with extensions", async function () {
        const res0 = await restaking.runAdminInitializeFragSOLTokenMint();
        expect(res0.fragSOLMint.address.toString()).eq(restaking.knownAddress.fragSOLTokenMint.toString());
        expect(res0.fragSOLMint.mintAuthority.toString()).eq(restaking.keychain.getKeypair('ADMIN').publicKey.toString()); // shall be transferred to a PDA below
        expect(res0.fragSOLMint.freezeAuthority).null;
    });

    step("update fragSOL token metadata", async function () {
        await restaking.runAdminUpdateTokenMetadata();
    });

    step("initialize fund accounts", async () => {
        const {fragSOLMint, fragSOLFundAccount} = await restaking.runAdminInitializeOrUpdateFundAccount();

        expect(fragSOLMint.mintAuthority.toString()).eq(restaking.knownAddress.fragSOLFund.toString());
        expect(fragSOLFundAccount.dataVersion).gt(1);
    })

    step("initialize reward accounts", async () => {
        const {fragSOLRewardAccount} = await restaking.runAdminInitializeOrUpdateRewardAccount();

        expect(fragSOLRewardAccount.dataVersion).eq(parseInt(restaking.getConstant('rewardAccountCurrentVersion')));
    })

    step("initialize fragSOL extra account meta list", async () => {
        const { fragSOLExtraAccountMetasAccount } = await restaking.runAdminInitializeFragSOLExtraAccountMetaList();

        expect(fragSOLExtraAccountMetasAccount.length).eq(8);
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

    step("create normalized token token mint", async function () {
        const {nSOLMint} = await restaking.runAdminInitializeNSOLTokenMint();
        expect(nSOLMint.address.toString()).eq(restaking.knownAddress.nSOLTokenMint.toString());
        expect(nSOLMint.mintAuthority.toString()).eq(restaking.keychain.getKeypair('ADMIN').publicKey.toString());
        expect(nSOLMint.freezeAuthority).null;
    })

    step("initialize normalized token pool", async () => {
        const {nSOLTokenPoolAccount} = await restaking.runAdminInitializeNormalizedTokenPoolAccounts();
        expect(nSOLTokenPoolAccount.normalizedTokenMint.toString()).eq(restaking.knownAddress.nSOLTokenMint.toString());
        expect(nSOLTokenPoolAccount.dataVersion).gt(1);
    })

    step("initialize normalized token pool supported tokens", async function () {
        const {nSOLTokenPool} = await restaking.runFundManagerInitializeNormalizeTokenPoolSupportedTokens();

        expect(nSOLTokenPool.supportedTokens.length).eq(Object.values(restaking.supportedTokenMetadata).length);
        let i = 0;
        for (const v of Object.values(restaking.supportedTokenMetadata)) {
            const supported = nSOLTokenPool.supportedTokens[i++];
            expect(supported.mint.toString()).eq(v.mint.toString());
            expect(supported.program.toString()).eq(v.program.toString());
            expect(supported.lockedAmount.toNumber()).eq(0);
        }
    });

    step("initialize fund supported tokens", async function () {
        const res0 = await restaking.runFundManagerInitializeFundSupportedTokens();

        expect(res0.fragSOLFund.numSupportedTokens).eq(Object.values(restaking.supportedTokenMetadata).length);
        let i = 0;
        for (const [symbol, v] of Object.entries(restaking.supportedTokenMetadata)) {
            const supported = res0.fragSOLFund.supportedTokens[i++];
            expect(supported.mint.toString()).eq(v.mint.toString());
            expect(supported.program.toString()).eq(v.program.toString());
            expect(supported.oneTokenAsSol.toNumber()).greaterThan(web3.LAMPORTS_PER_SOL).lessThan(2 * web3.LAMPORTS_PER_SOL);
            expect(supported.token.operationReservedAmount.toNumber()).eq(0);
        }
    });

    step("initialize fund normalized token", async () => {
        const {fragSOLFundAccount} = await restaking.runFundManagerInitializeFundNormalizedToken();
        expect(fragSOLFundAccount.normalizedToken.enabled).eq(1);
        expect(fragSOLFundAccount.normalizedToken.mint.toString()).eq(restaking.knownAddress.nSOLTokenMint.toString());
    });

    step("initialize fund fragSOL jito restaking vault", async () => {
        const {fragSOLFundJitoVRTAccount, fragSOLFundAccount} = await restaking.runFundManagerInitializeFundJitoRestakingVault();
        expect(fragSOLFundJitoVRTAccount.mint.toString()).eq(restaking.knownAddress.fragSOLJitoVRTMint.toString());
        expect(fragSOLFundJitoVRTAccount.owner.toString()).eq(restaking.knownAddress.fragSOLFund.toString());
        expect(fragSOLFundAccount.numRestakingVaults).eq(1);
        expect(fragSOLFundAccount.restakingVaults[0].vault.toString()).eq(restaking.knownAddress.fragSOLJitoVaultAccount.toString());
    });

    step("create new jito restaking vault with jitoSOL", async () => {
        await restaking.runAdminCreateNewJitoVault(restaking.supportedTokenMetadata.jitoSOL.mint, "jitoSOL");
    });

    step("initialize fund jitoSOL jito restaking vault", async () => {
        const vstMint = restaking.supportedTokenMetadata.jitoSOL.mint;
        await restaking.runFundManagerInitializeFundVSTJitoRestakingVault("jitoSOL", vstMint);
    });

    step("initialize fund, supported tokens, restaking vaults strategy", async () => {
        await restaking.runFundManagerUpdateFundConfigurations();
    });
});
