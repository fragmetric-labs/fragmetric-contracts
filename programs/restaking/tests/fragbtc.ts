import type { TestSuiteContext } from './utils';

export function initializeFragBTC(testCtx: TestSuiteContext) {
  const { validator, restaking, sdk } = testCtx;
  const { MAX_U64 } = sdk;

  const ctx = restaking.fragBTC;
  const initializationTasks = [
    // initialize receipt token mint and transfer hook metadata
    () =>
      ctx.initializeMint.execute({
        name: 'Fragmetric Staked BTC',
        symbol: 'fragBTC',
        uri: 'https://quicknode.quicknode-ipfs.com/ipfs/QmSxj4xv1LTgsAK3rHFEnEyfj6bC7tLWVtgjrtzAFbvxda',
        description: `fragBTC is redefining Bitcoin’s potential by enabling staking based returns.`,
        decimals: 8,
      }),
    () => ctx.initializeOrUpdateExtraAccountMetaList.execute(null),
    // initialize fund account and configuration
    () =>
      ctx.fund.initializeOrUpdateAccount.executeChained({ targetVersion: 18 }),
    () =>
      ctx.fund.updateGeneralStrategy.execute({
        depositEnabled: true,
        donationEnabled: true,
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
        mint: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
        pricingSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: '4yp9YAXCJsKWMDZq2Q4j4amktvJGXBCpr3Lmv2cYBrb8',
        },
      }),
    () =>
      ctx.fund.updateAssetStrategy.execute({
        tokenMint: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
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
    () =>
      ctx.fund.addSupportedToken.execute({
        mint: 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
        pricingSource: {
          __kind: 'PeggedToken',
          address: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
        },
      }),
    () =>
      ctx.fund.updateAssetStrategy.execute({
        tokenMint: 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
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
        mint: '11111111111111111111111111111111',
        decimals: 4,
        name: 'fPoint',
        description: 'Airdrop point for fToken',
      }),
    () => validator.skipSlots(1n),
    () =>
      ctx.reward.settleReward.execute({
        mint: '11111111111111111111111111111111',
        amount: 0n,
        isBonus: true,
      }),

    // initialize wrapped token mint and configuration
    () =>
      ctx.wrappedTokenMint.initializeMint.execute({
        mint: '9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN',
        name: 'Wrapped Fragmetric Staked BTC',
        symbol: 'wfragBTC',
        uri: 'https://quicknode.quicknode-ipfs.com/ipfs/Qmb1xtUcaPAZz5zHJRdLxEYtVdFJRdivYf1fG9ud3ivRVu',
        description: `wfragBTC is Solana's SPL token version of fragBTC that maximizes DeFi support and composability.`,
        decimals: 8,
      }),
    () =>
      ctx.fund.initializeWrappedToken.execute({
        mint: '9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN',
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
