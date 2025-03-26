import {BN, web3} from '@coral-xyz/anchor';
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";
import { RestakingPlayground } from '../../tools/restaking/playground';


describe("initialize", async () => {
    const restaking = await restakingPlayground as RestakingPlayground;

    step("try airdrop SOL to authorized wallets", async function () {
        await Promise.all([
            restaking.tryAirdrop(restaking.keychain.getPublicKey('ADMIN'), new BN(web3.LAMPORTS_PER_SOL).muln(100)),
            restaking.tryAirdrop(restaking.keychain.getPublicKey('FUND_MANAGER'), new BN(web3.LAMPORTS_PER_SOL).muln(100)),
        ]);
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
        await restaking.runAdminInitializeFragSOLExtraAccountMetaList();
        const { fragSOLExtraAccountMetasAccount } = await restaking.runAdminUpdateFragSOLExtraAccountMetaList();

        expect(fragSOLExtraAccountMetasAccount.length).eq(8);
    })

    step("initialize reward pools and rewards", async function () {
        const res0 = await restaking.runFundManagerInitializeRewardPools();

        expect(res0.fragSOLReward.dataVersion).eq(parseInt(restaking.getConstant('rewardAccountCurrentVersion')));

        expect(res0.fragSOLReward.numRewards).eq(Object.values(restaking.distributingRewardsMetadata).length);
        let i = 0;
        for (const v of Object.values(restaking.distributingRewardsMetadata)) {
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
        expect(res0.fragSOLReward.rewardPools1[res0.rewardPool.id].rewardSettlements1[0].numSettlementBlocks).eq(0);
        expect(res0.fragSOLReward.rewardPools1[res0.rewardPool.id].rewardSettlements1[0].settledAmount.toNumber()).eq(0);
        expect(res0.fragSOLReward.rewardPools1[res0.rewardPool.id].rewardSettlements1[0].settlementBlocksLastSlot.toNumber())
            .eq(res0.fragSOLReward.rewardPools1[res0.rewardPool.id].updatedSlot.toNumber());
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

    step("initialize fund jito restaking vault", async () => {
        const {fragSOLFund} = await restaking.runFundManagerInitializeFundJitoRestakingVaults();

        expect(fragSOLFund.numRestakingVaults).eq(Object.values(restaking.restakingVaultMetadata).length);
        let i = 0;
        for (const [symbol, v] of Object.entries(restaking.restakingVaultMetadata)) {
            const vault = fragSOLFund.restakingVaults[i++];
            expect(vault.vault.toString()).eq(v.vault.toString());
            expect(vault.program.toString()).eq(v.program.toString());
            expect(vault.supportedTokenMint.toString()).eq(v.VSTMint.toString());
            expect(vault.receiptTokenMint.toString()).eq(v.VRTMint.toString());
        }
    });

    step("initialize jito vault delegation at fund account", async function() {
        const {fragSOLFund} = await restaking.runFundManagerAddJitoRestakingVaultDelegations();

        Object.values(restaking.restakingVaultMetadata).forEach((v, i) => {
            const vault = fragSOLFund.restakingVaults[i];
            expect(vault.numDelegations).eq(Object.keys(v.operators).length);
            Object.values(v.operators).forEach((operator, i) => {
                expect(vault.delegations[i].operator.toString()).eq(operator.toString());
            })
        })
    });

    step("initialize fund, supported tokens, restaking vaults strategy", async () => {
        await restaking.runFundManagerUpdateFundConfigurations();
    });

    step("create wrapped token mint", async function () {
        const {wfragSOLMint} = await restaking.runAdminInitializeWfragSOLTokenMint();
        expect(wfragSOLMint.address.toString()).eq(restaking.knownAddress.wfragSOLTokenMint.toString());
        expect(wfragSOLMint.mintAuthority.toString()).eq(restaking.keychain.getPublicKey('ADMIN').toString());
        expect(wfragSOLMint.freezeAuthority).null;
    })

    step("initialize fund wrap account reward account", async () => {
        const {fragSOLFundWrapAccountRewardAccount} = await restaking.runAdminInitializeOrUpdateFundWrapAccountRewardAccount();
        expect(fragSOLFundWrapAccountRewardAccount.user.toString()).eq(restaking.knownAddress.fragSOLFundWrapAccount.toString());
        expect(fragSOLFundWrapAccountRewardAccount.dataVersion).gt(0);
    })

    step("initialize fund wrapped token", async () => {
        const {wfragSOLMint, fragSOLFundAccount} = await restaking.runFundManagerInitializeFundWrappedToken();
        expect(fragSOLFundAccount.wrappedToken.enabled).eq(1);
        expect(fragSOLFundAccount.wrappedToken.mint.toString()).eq(restaking.knownAddress.wfragSOLTokenMint.toString());
        expect(wfragSOLMint.mintAuthority.toString()).eq(restaking.knownAddress.fragSOLFund.toString());
    })

    step("add restaking vault compoundin reward tokens", async () => {
        const {fragSOLFund} = await restaking.runFundManagerAddRestakingVaultCompoundingRewardTokens();

        expect(fragSOLFund.numRestakingVaults).eq(Object.values(restaking.restakingVaultMetadata).length);
        Object.values(restaking.restakingVaultMetadata).forEach((vaultMetadata, i) => {
            const vault = fragSOLFund.restakingVaults[i];
            (vaultMetadata.compoundingRewards ?? []).forEach((rewardTokenMint, j) => {
                expect(vault.compoundingRewardTokenMints[j].toString()).eq(rewardTokenMint.toString());
            })
        });
    });

    step("add token swap strategies", async () => {
        if (restaking.tokenSwapStrategies.length == 0) {
            return;
        }

        const {fragSOLFund} = await restaking.runFundManagerAddTokenSwapStrategies();

        expect(fragSOLFund.numTokenSwapStrategies).eq(restaking.tokenSwapStrategies.length);
        Object.values(restaking.tokenSwapStrategies).forEach((strategyMetadata, i) => {
            const strategy = fragSOLFund.tokenSwapStrategies[i];
            expect(strategy.fromTokenMint.toString()).eq(strategyMetadata.fromTokenMint.toString());
            expect(strategy.toTokenMint.toString()).eq(strategyMetadata.toTokenMint.toString());
        })
    })
});
