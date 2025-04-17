import type { TestSuiteContext } from './utils';

export function initializeFragJTO(testCtx: TestSuiteContext) {
  const { validator, restaking, sdk } = testCtx;
  const { MAX_U64 } = sdk;

  const ctx = restaking.fragJTO;
  const initializationTasks = [
    // initialize receipt token mint and transfer hook metadata
    () =>
      ctx.initializeMint.execute({
        name: 'Fragmetric Staked JTO',
        symbol: 'fragJTO',
        uri: 'https://quicknode.quicknode-ipfs.com/ipfs/QmQyCKdba9f6dpxc43pGwQ66DvjpPFbE6S8rPrKDh1Sz72',
        description: `fragJTO is the staked Jito governance token that provides optimized restaking rewards.`,
        decimals: 9,
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
        withdrawalFeeRateBps: 10,
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
        mint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
        pricingSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: '2UhFnySoJi6c89aydGAGS7ZRemo2dbkFRhvSJqDX4gHJ',
        },
      }),
    () =>
      ctx.fund.updateAssetStrategy.execute({
        tokenMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
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
        mint: 'EAvS1wFjAccNpDYbAkW2dwUDEiC7BMvWzwUj2tjRUkHA',
        name: 'Wrapped Fragmetric Staked JTO',
        symbol: 'wfragJTO',
        uri: 'https://quicknode.quicknode-ipfs.com/ipfs/QmS4bSyX4v9tCWMnchJ7jhpWHy1YYKsD5nsMDAs45yX5hZ',
        description: `wfragJTO is Solana's SPL token version of fragJTO that maximizes DeFi support and composability.`,
        decimals: 9,
      }),
    () =>
      ctx.fund.initializeWrappedToken.execute({
        mint: 'EAvS1wFjAccNpDYbAkW2dwUDEiC7BMvWzwUj2tjRUkHA',
      }),
    () =>
      ctx.fund.addTokenSwapStrategy.execute({
        fromTokenMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
        toTokenMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
        swapSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: 'G2FiE1yn9N9ZJx5e1E2LxxMnHvb1H3hCuHLPfKJ98smA',
        },
      }),

    // initialize jito restaking vault (JTO)
    () =>
      ctx.fund.addRestakingVault.execute({
        vault: 'BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd',
        pricingSource: {
          __kind: 'JitoRestakingVault',
          address: 'BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd',
        },
      }),

    // configure reward settings
    () =>
      ctx.fund.addRestakingVaultCompoundingReward.execute({
        vault: 'BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd',
        rewardTokenMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
      }),
    () =>
      ctx.fund.addRestakingVaultDistributingReward.execute({
        vault: 'BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd',
        rewardTokenMint: 'REALSWTCH7J8JdafNBLZpfSCLiFwpMCqod2RpkU4RNn',
      }),

    // configure operator delegations
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd',
        operator: 'FzZ9EXmHv7ANCXijpALUBzCza6wYNprnsfaEHuoNx9sE', // Everstake
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd',
        operator: 'LKFpfXtBkH5b7D9mo8dPcjCLZCZpmLQC9ELkbkyVdah', // Luganodes
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd',
        operator: 'GZxp4e2Tm3Pw9GyAaxuF6odT3XkRM96jpZkp3nxhoK4Y', // PierTwo
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd',
        operator: 'CA8PaNSoFWzvbCJ2oK3QxBEutgyHSTT5omEptpj8YHPY', // Temporal
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd',
        operator: '7yofWXChEHkPTSnyFdKx2Smq5iWVbGB4P1dkdC6zHWYR', // ChorusOne
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd',
        operator: '29rxXT5zbTR1ctiooHtb1Sa1TD4odzhQHsrLz3D78G5w', // KILN
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd',
        operator: 'BFEsrxFPsBcY2hR5kgyfKnpwgEc8wYQdngvRukLQXwG2', // Helius
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd',
        operator: '2sHNuid4rus4sK2EmndLeZcPNKkgzuEoc8Vro3PH2qop', // Hashkey
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd',
        operator: '5TGRFaLy3eF93pSNiPamCgvZUN3gzdYcs7jA3iCAsd1L', // InfStones
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd',
        operator: '6AxtdRGAaiAyqcwxVBHsH3xtqCbQuffaiE4epT4koTxk', // Staked
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd',
        operator: 'C6AF8qGCo2dL815ziRCmfdbFeL5xbRLuSTSZzTGBH68y', // Figment
      }),

    () =>
      ctx.fund.updateRestakingVaultStrategy.executeChained({
        vault: 'BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd',
        delegations: [
          {
            operator: 'FzZ9EXmHv7ANCXijpALUBzCza6wYNprnsfaEHuoNx9sE',
            tokenAllocationWeight: 90n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: 'LKFpfXtBkH5b7D9mo8dPcjCLZCZpmLQC9ELkbkyVdah',
            tokenAllocationWeight: 90n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: 'GZxp4e2Tm3Pw9GyAaxuF6odT3XkRM96jpZkp3nxhoK4Y',
            tokenAllocationWeight: 90n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: 'CA8PaNSoFWzvbCJ2oK3QxBEutgyHSTT5omEptpj8YHPY',
            tokenAllocationWeight: 92n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: '7yofWXChEHkPTSnyFdKx2Smq5iWVbGB4P1dkdC6zHWYR',
            tokenAllocationWeight: 90n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: '29rxXT5zbTR1ctiooHtb1Sa1TD4odzhQHsrLz3D78G5w',
            tokenAllocationWeight: 90n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: 'BFEsrxFPsBcY2hR5kgyfKnpwgEc8wYQdngvRukLQXwG2',
            tokenAllocationWeight: 90n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: '2sHNuid4rus4sK2EmndLeZcPNKkgzuEoc8Vro3PH2qop',
            tokenAllocationWeight: 0n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: '5TGRFaLy3eF93pSNiPamCgvZUN3gzdYcs7jA3iCAsd1L',
            tokenAllocationWeight: 90n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: '6AxtdRGAaiAyqcwxVBHsH3xtqCbQuffaiE4epT4koTxk',
            tokenAllocationWeight: 80n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: 'C6AF8qGCo2dL815ziRCmfdbFeL5xbRLuSTSZzTGBH68y',
            tokenAllocationWeight: 100n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
        ],
        solAllocationCapacityAmount: MAX_U64,
        solAllocationWeight: 1n,
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
