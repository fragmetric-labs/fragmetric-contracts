import { TestSuiteContext } from '../../testutil';

export async function initializeFrag2(testCtx: TestSuiteContext) {
  const { validator, restaking, sdk } = testCtx;
  const { MAX_U64 } = sdk;

  const ctx = restaking.frag2;
  const initializationTasks = [
    // initialize receipt token mint and transfer hook metadata
    () =>
      ctx.initializeMint.execute({
        name: 'Fragmetric Squared',
        symbol: 'FRAG²',
        uri: 'https://quicknode.quicknode-ipfs.com/ipfs/Qma5yojojfZWATCRfHQnMn3iYyhko8RN7otKMWtnGspw2b',
        description: '',
        decimals: 9,
      }),
    () => ctx.initializeOrUpdateExtraAccountMetaList.execute(null),

    // initialize fund account and configuration
    () =>
      ctx.fund.initializeOrUpdateAccount.executeChained({ targetVersion: 19 }),
    () =>
      ctx.fund.updateGeneralStrategy.execute({
        depositEnabled: true,
        donationEnabled: false,
        transferEnabled: true,
        withdrawalEnabled: true,
        withdrawalBatchThresholdSeconds: 86400,
        withdrawalFeeRateBps: 20,
      }),
    () =>
      ctx.fund.updateGeneralStrategy.execute({
        withdrawalBatchThresholdSeconds: 1, // for fast local testing
      }),
    () =>
      ctx.fund.updateAssetStrategy.execute({
        tokenMint: null,
        solDepositable: false,
        solAccumulatedDepositAmount: null,
        solAccumulatedDepositCapacityAmount: MAX_U64,
        solWithdrawable: false,
        solWithdrawalNormalReserveRateBps: 0,
        solWithdrawalNormalReserveMaxAmount: MAX_U64,
      }),
    () =>
      ctx.fund.addSupportedToken.execute({
        mint: 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5',
        pricingSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: '4YH3gwg84qRHrPabYj4f7NMVawG5Wd6gM7ZDciuCckTo', // devnet: 7ro9kTyLX5sdKeQbw68fQB3ZZRpmoCRshoACngemey3b
        },
      }),
    () =>
      ctx.fund.updateAssetStrategy.execute({
        tokenMint: 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5',
        tokenDepositable: true,
        tokenAccumulatedDepositAmount: null,
        tokenAccumulatedDepositCapacityAmount: MAX_U64,
        tokenWithdrawable: true,
        tokenWithdrawalNormalReserveRateBps: 0,
        tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
        tokenRebalancingAmount: 0n,
        solAllocationWeight: 0n,
        solAllocationCapacityAmount: MAX_U64,
      }),

    // initialize reward account and configuration
    () =>
      ctx.reward.initializeOrUpdateAccount.executeChained({
        targetVersion: 35,
      }),
    () =>
      ctx.reward.addReward.execute({
        mint: 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
        decimals: 9,
        name: 'FVT',
        description: 'Governance vote distribution',
      }),
    () =>
      ctx.reward.updateReward.execute({
        mint: 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
        claimable: true,
      }),
    () => validator.skipSlots(1n),
    () =>
      ctx.reward.settleReward.execute({
        mint: 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
        amount: 0n,
      }),

    // initialize virtual vault
    () =>
      ctx.fund.addRestakingVault.execute({
        vault: '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
        pricingSource: {
          __kind: 'VirtualVault',
          address: '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
        },
      }),

    // configure reward settings
    () =>
      ctx.fund.addRestakingVaultCompoundingReward.execute({
        vault: '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
        rewardTokenMint: 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5',
      }),
    () =>
      ctx.fund.addRestakingVaultDistributingReward.execute({
        vault: '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
        rewardTokenMint: 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
      }),
    () =>
      ctx.fund.updateRestakingVaultDistributingRewardHarvestThreshold.execute({
        vault: '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
        rewardTokenMint: 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
        harvestThresholdMinAmount: 1_000_000_000n,
        harvestThresholdMaxAmount: 1_000_000_000_000n,
        harvestThresholdIntervalSeconds: 1n,
      }),

    // initialize address lookup table
    () =>
      ctx.fund.addressLookupTable
        .resolveFrequentlyUsedAddresses()
        .then((addresses) =>
          ctx.fund.addressLookupTable.initializeOrUpdateAccount.executeChained({
            addresses,
          })
        ),

    // wait for two slots to activate ALT
    () => validator.skipSlots(2n),
  ].reduce(
    async (prevLogs, task) => {
      const logs = await prevLogs;
      const ctx = await task();
      if (ctx?.result?.meta?.logMessages) {
        logs.push(ctx.result.meta.logMessages);
      }
      return logs;
    },
    Promise.resolve([] as (readonly string[])[])
  );

  return { ...testCtx, initializationTasks };
}
