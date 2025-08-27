import { TestSuiteContext } from '../../testutil';

export async function initializeFragSwitch(testCtx: TestSuiteContext) {
  const { validator, restaking, sdk } = testCtx;
  const { MAX_U64 } = sdk;

  const ctx = restaking.fragSWTCH;
  const initializationTasks = [
    // initialize receipt token mint and transfer hook metadata
    () =>
      ctx.initializeMint.execute({
        name: 'Fragmetric Staked SWTCH',
        symbol: 'fragSWTCH',
        uri: 'https://quicknode.quicknode-ipfs.com/ipfs/QmTLAXhfLsrF3ah2C6u17hHcGSqmfb2VZta3rumFRQ7AHj',
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
        operationEnabled: true,
        withdrawalBatchThresholdSeconds: 86400,
        withdrawalFeeRateBps: 20,
      }),
    () =>
      // for fast local testing
      ctx.fund.updateGeneralStrategy.execute({
        withdrawalBatchThresholdSeconds: 1,
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
        mint: 'SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx', // devnet: FSWSBMV5EB7J8JdafNBLZpfSCLiFwpMCqod2RpkU4RNn
        pricingSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: '6urvuvb6XxjqVgCzEaHMTVJ8PkS2PQ91at455Vgt89ME', // devnet: HLD9ZAsUNPXUjHfJ34UpVGg8HcjB6FcaSTGzrLHPdgFK
        },
      }),
    () =>
      ctx.fund.updateAssetStrategy.execute({
        tokenMint: 'SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx',
        tokenDepositable: true,
        tokenAccumulatedDepositAmount: null,
        tokenAccumulatedDepositCapacityAmount: MAX_U64,
        tokenWithdrawable: true,
        tokenWithdrawalNormalReserveRateBps: 0,
        tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
        solAllocationWeight: 0n,
        solAllocationCapacityAmount: MAX_U64,
      }),

    // initialize reward account and configuration
    () =>
      ctx.reward.initializeOrUpdateAccount.executeChained({
        targetVersion: 35,
      }),

    // initialize virtual vault
    () =>
      ctx.fund.addRestakingVault.execute({
        vault: '9oDtsX1hoKCG31VAVKCEmTEEbjgDa6fq6vVHyWAVzWV4', // devnet: HfA2hk1cYDXbaMMMhaSGGhXXKgEBmbdvX9jQGS319mng
        pricingSource: {
          __kind: 'VirtualVault',
          address: '9oDtsX1hoKCG31VAVKCEmTEEbjgDa6fq6vVHyWAVzWV4',
        },
      }),

    // configure reward settings
    () =>
      ctx.fund.addRestakingVaultCompoundingReward.execute({
        vault: '9oDtsX1hoKCG31VAVKCEmTEEbjgDa6fq6vVHyWAVzWV4',
        rewardTokenMint: 'SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx',
      }),
    () =>
      ctx.fund.updateRestakingVaultRewardHarvestThreshold.execute({
        vault: '9oDtsX1hoKCG31VAVKCEmTEEbjgDa6fq6vVHyWAVzWV4',
        rewardTokenMint: 'SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx',
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
