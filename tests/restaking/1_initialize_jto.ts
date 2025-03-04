import {BN, web3} from '@coral-xyz/anchor';
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";
import { RestakingPlayground } from '../../tools/restaking/jto_playground';


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

    step("create fragJTO token mint with extensions", async function () {
        const res0 = await restaking.runAdminInitializeFragJTOTokenMint();
        expect(res0.fragJTOMint.address.toString()).eq(restaking.knownAddress.fragJTOTokenMint.toString());
        expect(res0.fragJTOMint.mintAuthority.toString()).eq(restaking.keychain.getKeypair('ADMIN').publicKey.toString()); // shall be transferred to a PDA below
        expect(res0.fragJTOMint.freezeAuthority).null;
    });

    step("update fragJTO token metadata", async function () {
        await restaking.runAdminUpdateTokenMetadata();
    });

    step("initialize fund accounts", async () => {
        const {fragJTOMint, fragJTOFundAccount} = await restaking.runAdminInitializeOrUpdateFundAccount();

        expect(fragJTOMint.mintAuthority.toString()).eq(restaking.knownAddress.fragJTOFund.toString());
        expect(fragJTOFundAccount.dataVersion).gt(1);
    })

    step("initialize reward accounts", async () => {
        const {fragJTORewardAccount} = await restaking.runAdminInitializeOrUpdateRewardAccount();

        expect(fragJTORewardAccount.dataVersion).eq(parseInt(restaking.getConstant('rewardAccountCurrentVersion')));
    })

    step("initialize fragJTO extra account meta list", async () => {
        await restaking.runAdminInitializeFragJTOExtraAccountMetaList();
        const { fragJTOExtraAccountMetasAccount } = await restaking.runAdminUpdateFragJTOExtraAccountMetaList();

        expect(fragJTOExtraAccountMetasAccount.length).eq(8);
    })

    step("initialize reward pools and rewards", async function () {
        const res0 = await restaking.runFundManagerInitializeRewardPools();

        expect(res0.fragJTOReward.dataVersion).eq(parseInt(restaking.getConstant('rewardAccountCurrentVersion')));

        expect(res0.fragJTOReward.numRewards).eq(Object.values(restaking.distributingRewardsMetadata).length);
        let i = 0;
        for (const v of Object.values(restaking.distributingRewardsMetadata)) {
            const reward = res0.fragJTOReward.rewards1[i++];
            expect(restaking.binToString(reward.name)).eq(v.name.toString());
            expect(restaking.binToString(reward.description)).eq(v.description.toString());
        }

        expect(res0.fragJTOReward.numRewardPools).eq(Object.values(restaking.rewardPoolsMetadata).length);
        i = 0;
        for (const v of Object.values(restaking.rewardPoolsMetadata)) {
            const pool = res0.fragJTOReward.rewardPools1[i++];
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
        expect(res0.fragJTOReward.rewardPools1[res0.rewardPool.id].numRewardSettlements).eq(1);
        expect(res0.fragJTOReward.rewardPools1[res0.rewardPool.id].rewardSettlements1[0].rewardId).eq(res0.reward.id);
        expect(res0.fragJTOReward.rewardPools1[res0.rewardPool.id].rewardSettlements1[0].rewardPoolId).eq(res0.rewardPool.id);
        expect(res0.fragJTOReward.rewardPools1[res0.rewardPool.id].rewardSettlements1[0].numSettlementBlocks).eq(1);
        expect(res0.fragJTOReward.rewardPools1[res0.rewardPool.id].rewardSettlements1[0].settledAmount.toNumber()).eq(0);
    });

    step("initialize fund supported tokens", async function () {
        const res0 = await restaking.runFundManagerInitializeFundSupportedTokens();

        expect(res0.fragJTOFund.numSupportedTokens).eq(Object.values(restaking.supportedTokenMetadata).length);
        let i = 0;
        for (const [symbol, v] of Object.entries(restaking.supportedTokenMetadata)) {
            const supported = res0.fragJTOFund.supportedTokens[i++];
            expect(supported.mint.toString()).eq(v.mint.toString());
            expect(supported.program.toString()).eq(v.program.toString());
            expect(supported.token.operationReservedAmount.toNumber()).eq(0);
        }
    });

    step("initialize fund jito restaking vault", async () => {
        await Promise.all(Object.values(restaking.restakingVaultMetadata).map(v => restaking.runAdminSetSecondaryAdminForJitoVault(v.vault)));
        const {fragJTOFund} = await restaking.runFundManagerInitializeFundJitoRestakingVaults();

        expect(fragJTOFund.numRestakingVaults).eq(Object.values(restaking.restakingVaultMetadata).length);
        let i = 0;
        for (const [symbol, v] of Object.entries(restaking.restakingVaultMetadata)) {
            const vault = fragJTOFund.restakingVaults[i++];
            expect(vault.vault.toString()).eq(v.vault.toString());
            expect(vault.program.toString()).eq(v.program.toString());
            expect(vault.supportedTokenMint.toString()).eq(v.VSTMint.toString());
            expect(vault.receiptTokenMint.toString()).eq(v.VRTMint.toString());
        }
    });

    step("initialize vault delegation at fund account", async function() {
        await Promise.all(
            Object.values(restaking.restakingVaultMetadata).flatMap(vault => {
                return vault.operators.map(operator => {
                    return restaking.runFundManagerAddJitoRestakingVaultDelegation(vault.vault, operator);
                })
            }),
        );
    });

    step("initialize fund, supported tokens, restaking vaults strategy", async () => {
        await restaking.runFundManagerUpdateFundConfigurations();
    });

    step("create wrapped token mint", async function () {
        const {wfragJTOMint} = await restaking.runAdminInitializeWfragJTOTokenMint();
        expect(wfragJTOMint.address.toString()).eq(restaking.knownAddress.wfragJTOTokenMint.toString());
        expect(wfragJTOMint.mintAuthority.toString()).eq(restaking.keychain.getPublicKey('ADMIN').toString());
        expect(wfragJTOMint.freezeAuthority).null;
    })

    step("initialize fund wrap account reward account", async () => {
        const {fragJTOFundWrapAccountRewardAccount} = await restaking.runAdminInitializeOrUpdateFundWrapAccountRewardAccount();
        expect(fragJTOFundWrapAccountRewardAccount.user.toString()).eq(restaking.knownAddress.fragJTOFundWrapAccount.toString());
        expect(fragJTOFundWrapAccountRewardAccount.dataVersion).gt(0);
    })

    step("initialize fund wrapped token", async () => {
        const {wfragJTOMint, fragJTOFundAccount} = await restaking.runFundManagerInitializeFundWrappedToken();
        expect(fragJTOFundAccount.wrappedToken.enabled).eq(1);
        expect(fragJTOFundAccount.wrappedToken.mint.toString()).eq(restaking.knownAddress.wfragJTOTokenMint.toString());
        expect(wfragJTOMint.mintAuthority.toString()).eq(restaking.knownAddress.fragJTOFund.toString());
    })

    step("add restaking vault compoundin reward tokens", async () => {
        const {fragJTOFund} = await restaking.runFundManagerAddRestakingVaultCompoundingRewardTokens();

        expect(fragJTOFund.numRestakingVaults).eq(Object.values(restaking.restakingVaultMetadata).length);
        Object.values(restaking.restakingVaultMetadata).forEach((vaultMetadata, i) => {
            const vault = fragJTOFund.restakingVaults[i];
            (vaultMetadata.compoundingRewards ?? []).forEach((rewardTokenMint, j) => {
                expect(vault.compoundingRewardTokenMints[j].toString()).eq(rewardTokenMint.toString());
            })
        });
    });

    step("add token swap strategies", async () => {
        const {fragJTOFund} = await restaking.runFundManagerAddTokenSwapStrategies();

        expect(fragJTOFund.numTokenSwapStrategies).eq(Object.values(restaking.tokenSwapStrategies).length);
        Object.values(restaking.tokenSwapStrategies).forEach((strategyMetadata, i) => {
            const strategy = fragJTOFund.tokenSwapStrategies[i];
            expect(strategy.fromTokenMint.toString()).eq(strategyMetadata.fromTokenMint.toString());
            expect(strategy.toTokenMint.toString()).eq(strategyMetadata.toTokenMint.toString());
        })
    })
});
