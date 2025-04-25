import type { RestakingProgram } from '@fragmetric-labs/sdk';
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
      ctx.fund.initializeOrUpdateAccount.executeChained({ targetVersion: 18 }),
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

    // initialize Solv BTC vault (zBTC)
    async () =>
      solv.zBTC.initializeMint.execute({
        // temporarily set VRT mint authority to fund manager
        authority: (ctx.program as RestakingProgram).knownAddresses.fundManager,
      }),
    async () => {
      const [vault, vrt, vst] = await Promise.all([
        solv.zBTC.vault.resolveAddress(),
        solv.zBTC.resolveAddress(),
        solv.zBTC.supportedTokenMint.resolveAddress(),
      ]);
      return ctx.fund.addRestakingVault.execute({
        vault: vault!,
        pricingSource: {
          __kind: 'SolvBTCVault',
          address: vault!,
        },
        temp: {
          vrt: vrt!,
          vst: vst!,
        },
      });
    },

    // configure reward settings (zBTC vault)
    async () =>
      ctx.fund.addRestakingVaultDistributingReward.execute({
        vault: (await solv.zBTC.vault.resolveAddress())!,
        rewardTokenMint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
      }),
    async () =>
      ctx.reward.addReward.execute({
        mint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
        decimals: 6,
        name: 'ZEUS',
        description: 'ZEUS Incentive',
      }),
    async () =>
      ctx.reward.updateReward.execute({
        mint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
        claimable: true,
      }),

    // initialize Solv BTC vault (cbBTC)
    async () =>
      solv.cbBTC.initializeMint.execute({
        // temporarily set VRT mint authority to fund manager
        authority: (ctx.program as RestakingProgram).knownAddresses.fundManager,
      }),
    async () => {
      const [vault, vrt, vst] = await Promise.all([
        solv.cbBTC.vault.resolveAddress(),
        solv.cbBTC.resolveAddress(),
        solv.cbBTC.supportedTokenMint.resolveAddress(),
      ]);
      return ctx.fund.addRestakingVault.execute({
        vault: vault!,
        pricingSource: {
          __kind: 'SolvBTCVault',
          address: vault!,
        },
        temp: {
          vrt: vrt!,
          vst: vst!,
        },
      });
    },

    // initialize Solv BTC vault (wBTC)
    async () =>
      solv.wBTC.initializeMint.execute({
        // temporarily set VRT mint authority to fund manager
        authority: (ctx.program as RestakingProgram).knownAddresses.fundManager,
      }),
    async () => {
      const [vault, vrt, vst] = await Promise.all([
        solv.wBTC.vault.resolveAddress(),
        solv.wBTC.resolveAddress(),
        solv.wBTC.supportedTokenMint.resolveAddress(),
      ]);
      return ctx.fund.addRestakingVault.execute({
        vault: vault!,
        pricingSource: {
          __kind: 'SolvBTCVault',
          address: vault!,
        },
        temp: {
          vrt: vrt!,
          vst: vst!,
        },
      });
    },

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
