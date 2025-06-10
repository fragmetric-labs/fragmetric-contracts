import type { TestSuiteContext } from '../../testutil';

export function initializeFragBTC(testCtx: TestSuiteContext) {
  const { validator, restaking, solv, sdk } = testCtx;
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
        tokenWithdrawalNormalReserveRateBps: 30, // 0.3%
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
        tokenWithdrawalNormalReserveRateBps: 30, // 0.3%
        tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
        tokenRebalancingAmount: 0n,
        solAllocationWeight: 0n,
        solAllocationCapacityAmount: MAX_U64,
      }),
    () =>
      ctx.fund.addSupportedToken.execute({
        mint: '3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh',
        pricingSource: {
          __kind: 'PeggedToken',
          address: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
        },
      }),
    () =>
      ctx.fund.updateAssetStrategy.execute({
        tokenMint: '3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh',
        tokenDepositable: true,
        tokenAccumulatedDepositAmount: null,
        tokenAccumulatedDepositCapacityAmount: MAX_U64,
        tokenWithdrawable: true,
        tokenWithdrawalNormalReserveRateBps: 30, // 0.3%
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

    // initialize Solv BTC vault (zBTC)
    () => solv.zBTC.initializeReceiptTokenMint.execute(null),
    () => solv.zBTC.initializeOrUpdateAccount.execute(null),
    async () => {
      await solv.zBTC.resolve(true);
      return ctx.fund.addRestakingVault.execute({
        vault: solv.zBTC.address!,
        pricingSource: {
          __kind: 'SolvBTCVault',
          address: solv.zBTC.address!,
        },
      });
    },
    () =>
      ctx.fund.updateRestakingVaultStrategy.execute({
        vault: solv.zBTC.address!,
        solAllocationCapacityAmount: MAX_U64,
        solAllocationWeight: 1n,
      }),

    // configure reward settings (zBTC vault)
    () =>
      solv.zBTC.delegateRewardTokenAccount.execute({
        mint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
        delegate: ctx.fund.address!,
      }),
    async () =>
      ctx.fund.addRestakingVaultDistributingReward.execute({
        vault: (await solv.zBTC.resolveAddress())!,
        rewardTokenMint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
      }),
    () =>
      ctx.reward.addReward.execute({
        mint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
        decimals: 6,
        name: 'ZEUS',
        description: 'ZEUS incentive',
      }),
    () =>
      ctx.reward.updateReward.execute({
        mint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
        claimable: true,
      }),
    () =>
      ctx.reward.settleReward.execute({
        mint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
        amount: 0n,
      }),

    // initialize Solv BTC vault (cbBTC)
    () => solv.cbBTC.initializeReceiptTokenMint.execute(null),
    () => solv.cbBTC.initializeOrUpdateAccount.execute(null),
    async () => {
      await solv.cbBTC.resolve(true);
      return ctx.fund.addRestakingVault.execute({
        vault: solv.cbBTC.address!,
        pricingSource: {
          __kind: 'SolvBTCVault',
          address: solv.cbBTC.address!,
        },
      });
    },
    () =>
      ctx.fund.updateRestakingVaultStrategy.execute({
        vault: solv.cbBTC.address!,
        solAllocationCapacityAmount: MAX_U64,
        solAllocationWeight: 1n,
      }),

    // initialize Solv BTC vault (wBTC)
    () => solv.wBTC.initializeReceiptTokenMint.execute(null),
    () => solv.wBTC.initializeOrUpdateAccount.execute(null),
    async () => {
      await solv.wBTC.resolve(true);
      return ctx.fund.addRestakingVault.execute({
        vault: solv.wBTC.address!,
        pricingSource: {
          __kind: 'SolvBTCVault',
          address: solv.wBTC.address!,
        },
      });
    },
    () =>
      ctx.fund.updateRestakingVaultStrategy.execute({
        vault: solv.wBTC.address!,
        solAllocationCapacityAmount: MAX_U64,
        solAllocationWeight: 1n,
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
