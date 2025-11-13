import type { TestSuiteContext } from '../../testutil';

export function initializeFragUSD(testCtx: TestSuiteContext) {
  const { validator, restaking, sdk } = testCtx;
  const { MAX_U64 } = sdk;

  const ctx = restaking.fragUSD;
  const initializationTasks = [
    // initialize receipt token mint and transfer hook metadata
    () =>
      ctx.initializeMint.execute({
        name: 'Fragmetric Staked USDC',
        symbol: 'fragUSD',
        uri: '', // TODO: Update uri
        description: 'fragUSD is ...', // TODO: Update description
        decimals: 6,
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
        withdrawalFeeRateBps: 100,
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
        mint: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v',
        pricingSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: 'Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE',
        },
      }),
    () =>
      ctx.fund.updateAssetStrategy.execute({
        tokenMint: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v',
        tokenDepositable: true,
        tokenAccumulatedDepositAmount: null,
        tokenAccumulatedDepositCapacityAmount: MAX_U64,
        tokenWithdrawable: true,
        tokenWithdrawalNormalReserveRateBps: 30,
        tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
        solAllocationWeight: 0n,
        solAllocationCapacityAmount: MAX_U64,
      }),
    // initialize reward account and configuration
    () =>
      ctx.reward.initializeOrUpdateAccount.executeChained({
        targetVersion: 35,
      }),
    // initialize wrapped token mint and configuration
    () =>
      ctx.wrappedTokenMint.initializeMint.execute({
        mint: '6HdgTPBRV8KvdP6qy3S81ZFBHyTWmozg7ENy1GCrRyfc',
        name: 'Wrapped Fragmetric Staked USDC',
        symbol: 'wfragUSD',
        uri: '', // TODO: Update uri
        description: '', // TODO: Update description
        decimals: 6,
      }),
    () =>
      ctx.fund.initializeWrappedToken.execute({
        mint: '6HdgTPBRV8KvdP6qy3S81ZFBHyTWmozg7ENy1GCrRyfc',
      }),
    () =>
      ctx.fund.addRestakingVault.execute({
        vault: 'CoHd9JpwfcA76XQGA4AYfnjvAtWKoBQ6eWBkFzR1A2ui', // TODO: This address need to be replaced
        pricingSource: {
          __kind: 'DriftVault',
          address: 'CoHd9JpwfcA76XQGA4AYfnjvAtWKoBQ6eWBkFzR1A2ui', // TODO: This address need to be replaced
        },
      }),
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
