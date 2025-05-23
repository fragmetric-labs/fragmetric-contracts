import type { TestSuiteContext } from '../../testutil';

export function initializeFragSOL(testCtx: TestSuiteContext) {
  const { validator, restaking, sdk } = testCtx;
  const { MAX_U64 } = sdk;

  const ctx = restaking.fragSOL;
  const initializationTasks = [
    // initialize receipt token mint and transfer hook metadata
    () =>
      ctx.initializeMint.execute({
        name: 'Fragmetric Restaked SOL',
        symbol: 'fragSOL',
        uri: 'https://quicknode.quicknode-ipfs.com/ipfs/QmcueajXkNzoYRhcCv323PMC8VVGiDvXaaVXkMyYcyUSRw',
        description: `fragSOL is Solana's first native LRT that provides optimized restaking rewards.`,
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
        withdrawalFeeRateBps: 20,
      }),
    () =>
      ctx.fund.updateGeneralStrategy.execute({
        withdrawalBatchThresholdSeconds: 1, // for fast local testing
      }),
    () =>
      ctx.fund.updateAssetStrategy.execute({
        tokenMint: null,
        solDepositable: true,
        solAccumulatedDepositAmount: null,
        solAccumulatedDepositCapacityAmount: MAX_U64,
        solWithdrawable: true,
        solWithdrawalNormalReserveRateBps: 0,
        solWithdrawalNormalReserveMaxAmount: MAX_U64,
      }),
    // NOTE: bSOL is no longer supported
    // () =>
    //   ctx.fund.addSupportedToken.execute({
    //     mint: 'bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1',
    //     pricingSource: {
    //       __kind: 'SPLStakePool',
    //       address: 'stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi',
    //     },
    //   }),
    // () =>
    //   ctx.fund.updateAssetStrategy.execute({
    //     tokenMint: 'bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1',
    //     tokenDepositable: false,
    //     tokenAccumulatedDepositAmount: null,
    //     tokenAccumulatedDepositCapacityAmount: MAX_U64,
    //     tokenWithdrawable: false,
    //     tokenWithdrawalNormalReserveRateBps: 0,
    //     tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
    //     tokenRebalancingAmount: 0n,
    //     solAllocationWeight: 0n,
    //     solAllocationCapacityAmount: MAX_U64,
    //   }),
    () =>
      ctx.fund.addSupportedToken.execute({
        mint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
        pricingSource: {
          __kind: 'SPLStakePool',
          address: 'Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb',
        },
      }),
    () =>
      ctx.fund.updateAssetStrategy.execute({
        tokenMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
        tokenDepositable: true,
        tokenAccumulatedDepositAmount: null,
        tokenAccumulatedDepositCapacityAmount: MAX_U64,
        tokenWithdrawable: false,
        tokenWithdrawalNormalReserveRateBps: 0,
        tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
        tokenRebalancingAmount: 0n,
        solAllocationWeight: 1n,
        solAllocationCapacityAmount: MAX_U64,
      }),
    () =>
      ctx.fund.addSupportedToken.execute({
        mint: 'mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So',
        pricingSource: {
          __kind: 'MarinadeStakePool',
          address: '8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC',
        },
      }),
    () =>
      ctx.fund.updateAssetStrategy.execute({
        tokenMint: 'mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So',
        tokenDepositable: false,
        tokenAccumulatedDepositAmount: null,
        tokenAccumulatedDepositCapacityAmount: MAX_U64,
        tokenWithdrawable: false,
        tokenWithdrawalNormalReserveRateBps: 0,
        tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
        tokenRebalancingAmount: 0n,
        solAllocationWeight: 0n,
        solAllocationCapacityAmount: MAX_U64,
      }),
    () =>
      ctx.fund.addSupportedToken.execute({
        mint: 'BNso1VUJnh4zcfpZa6986Ea66P6TCp59hvtNJ8b1X85',
        pricingSource: {
          __kind: 'SPLStakePool',
          address: 'Hr9pzexrBge3vgmBNRR8u42CNQgBXdHm4UkUN2DH4a7r',
        },
      }),
    () =>
      ctx.fund.updateAssetStrategy.execute({
        tokenMint: 'BNso1VUJnh4zcfpZa6986Ea66P6TCp59hvtNJ8b1X85',
        tokenDepositable: false,
        tokenAccumulatedDepositAmount: null,
        tokenAccumulatedDepositCapacityAmount: MAX_U64,
        tokenWithdrawable: false,
        tokenWithdrawalNormalReserveRateBps: 0,
        tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
        tokenRebalancingAmount: 0n,
        solAllocationWeight: 0n,
        solAllocationCapacityAmount: MAX_U64,
      }),
    () =>
      ctx.fund.addSupportedToken.execute({
        mint: 'Bybit2vBJGhPF52GBdNaQfUJ6ZpThSgHBobjWZpLPb4B',
        pricingSource: {
          __kind: 'SanctumSingleValidatorSPLStakePool',
          address: '2aMLkB5p5gVvCwKkdSo5eZAL1WwhZbxezQr1wxiynRhq',
        },
      }),
    () =>
      ctx.fund.updateAssetStrategy.execute({
        tokenMint: 'Bybit2vBJGhPF52GBdNaQfUJ6ZpThSgHBobjWZpLPb4B',
        tokenDepositable: true,
        tokenAccumulatedDepositAmount: null,
        tokenAccumulatedDepositCapacityAmount: MAX_U64,
        tokenWithdrawable: false,
        tokenWithdrawalNormalReserveRateBps: 0,
        tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
        tokenRebalancingAmount: 0n,
        solAllocationWeight: 0n,
        solAllocationCapacityAmount: MAX_U64,
      }),
    () =>
      ctx.fund.addSupportedToken.execute({
        mint: 'jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v',
        pricingSource: {
          __kind: 'SanctumMultiValidatorSPLStakePool',
          address: '8VpRhuxa7sUUepdY3kQiTmX9rS5vx4WgaXiAnXq4KCtr',
        },
      }),
    () =>
      ctx.fund.updateAssetStrategy.execute({
        tokenMint: 'jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v',
        tokenDepositable: false,
        tokenAccumulatedDepositAmount: null,
        tokenAccumulatedDepositCapacityAmount: MAX_U64,
        tokenWithdrawable: false,
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
    () =>
      ctx.reward.addReward.execute({
        mint: 'FSWSBMV5EB7J8JdafNBLZpfSCLiFwpMCqod2RpkU4RNn',
        decimals: 9,
        name: 'SWTCH',
        description: 'Switchboard Token',
      }),
    () => validator.skipSlots(1n),
    () =>
      ctx.reward.settleReward.execute({
        mint: '11111111111111111111111111111111',
        amount: 0n,
        isBonus: true,
      }),
    // NOTE: updateReward ix test
    // () => receiptToken.reward.updateRewardTransaction.execute({
    //   mint: 'FSWSBMV5EB7J8JdafNBLZpfSCLiFwpMCqod2RpkU4RNn',
    //   newMint: 'REALSWTCH7J8JdafNBLZpfSCLiFwpMCqod2RpkU4RNn',
    //   newDecimals: 9,
    //   claimable: true,
    // }),
    () =>
      ctx.reward.settleReward.execute({
        mint: 'FSWSBMV5EB7J8JdafNBLZpfSCLiFwpMCqod2RpkU4RNn',
        amount: 0n,
      }),

    // initialize normalized token pool and configuration
    () =>
      ctx.normalizedTokenMint.initializeMint.execute({
        mint: '4noNmx2RpxK4zdr68Fq1CYM5VhN4yjgGZEFyuB7t2pBX',
        name: 'Normalized Liquid Staked Solana',
        symbol: 'nSOL',
        uri: 'https://quicknode.quicknode-ipfs.com/ipfs/QmR5pP6Zo65XWCEXgixY8UtZjWbYPKmYHcyxzUq4p1KZt5',
        description: `fragSOL is Solana's first native LRT that provides optimized restaking rewards.`,
        decimals: 9,
      }),
    () =>
      ctx.fund.initializeNormalizedToken.execute({
        mint: '4noNmx2RpxK4zdr68Fq1CYM5VhN4yjgGZEFyuB7t2pBX',
      }),
    // NOTE: bSOL is no longer supported
    // () =>
    //   ctx.normalizedTokenPool.addSupportedToken.execute({
    //     mint: 'bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1',
    //     pricingSource: {
    //       __kind: 'SPLStakePool',
    //       address: 'stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi',
    //     },
    //   }),
    () =>
      ctx.normalizedTokenPool.addSupportedToken.execute({
        mint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
        pricingSource: {
          __kind: 'SPLStakePool',
          address: 'Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb',
        },
      }),
    () =>
      ctx.normalizedTokenPool.addSupportedToken.execute({
        mint: 'mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So',
        pricingSource: {
          __kind: 'MarinadeStakePool',
          address: '8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC',
        },
      }),
    () =>
      ctx.normalizedTokenPool.addSupportedToken.execute({
        mint: 'BNso1VUJnh4zcfpZa6986Ea66P6TCp59hvtNJ8b1X85',
        pricingSource: {
          __kind: 'SPLStakePool',
          address: 'Hr9pzexrBge3vgmBNRR8u42CNQgBXdHm4UkUN2DH4a7r',
        },
      }),
    () =>
      ctx.normalizedTokenPool.addSupportedToken.execute({
        mint: 'Bybit2vBJGhPF52GBdNaQfUJ6ZpThSgHBobjWZpLPb4B',
        pricingSource: {
          __kind: 'SanctumSingleValidatorSPLStakePool',
          address: '2aMLkB5p5gVvCwKkdSo5eZAL1WwhZbxezQr1wxiynRhq',
        },
      }),
    () =>
      ctx.normalizedTokenPool.addSupportedToken.execute({
        mint: 'jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v',
        pricingSource: {
          __kind: 'SanctumMultiValidatorSPLStakePool',
          address: '8VpRhuxa7sUUepdY3kQiTmX9rS5vx4WgaXiAnXq4KCtr',
        },
      }),
    // initialize wrapped token mint and configuration
    () =>
      ctx.wrappedTokenMint.initializeMint.execute({
        mint: 'h7veGmqGWmFPe2vbsrKVNARvucfZ2WKCXUvJBmbJ86Q',
        name: 'Wrapped Fragmetric Restaked SOL',
        symbol: 'wfragSOL',
        uri: 'https://quicknode.quicknode-ipfs.com/ipfs/QmaTVVmyvbJXs2Rqcqs76N5UiuPZ2iKCKrb5BpyB13vwzU',
        description: `wfragSOL is Solana's SPL token version of fragSOL that maximizes DeFi support and composability.`,
        decimals: 9,
      }),
    () =>
      ctx.fund.initializeWrappedToken.execute({
        mint: 'h7veGmqGWmFPe2vbsrKVNARvucfZ2WKCXUvJBmbJ86Q',
      }),

    // initialize jito restaking vault (nSOL)
    () =>
      ctx.fund.addRestakingVault.execute({
        vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
        pricingSource: {
          __kind: 'JitoRestakingVault',
          address: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
        },
      }),
    () =>
      ctx.fund.updateRestakingVaultStrategy.execute({
        vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
        solAllocationWeight: 100n,
        solAllocationCapacityAmount: MAX_U64,
      }),

    // configure reward settings
    () =>
      ctx.fund.addRestakingVaultCompoundingReward.execute({
        vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
        rewardTokenMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
      }),
    // () =>
    //   ctx.fund.addRestakingVaultDistributingReward.execute({
    //     vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
    //     rewardTokenMint: 'REALSWTCH7J8JdafNBLZpfSCLiFwpMCqod2RpkU4RNn',
    //   }),

    // configure operator delegations
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
        operator: 'FzZ9EXmHv7ANCXijpALUBzCza6wYNprnsfaEHuoNx9sE', // Everstake
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
        operator: '29rxXT5zbTR1ctiooHtb1Sa1TD4odzhQHsrLz3D78G5w', // KILN
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
        operator: 'LKFpfXtBkH5b7D9mo8dPcjCLZCZpmLQC9ELkbkyVdah', // Luganodes
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
        operator: 'GZxp4e2Tm3Pw9GyAaxuF6odT3XkRM96jpZkp3nxhoK4Y', // PierTwo
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
        operator: 'CA8PaNSoFWzvbCJ2oK3QxBEutgyHSTT5omEptpj8YHPY', // Temporal
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
        operator: '7yofWXChEHkPTSnyFdKx2Smq5iWVbGB4P1dkdC6zHWYR', // ChorusOne
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
        operator: 'BFEsrxFPsBcY2hR5kgyfKnpwgEc8wYQdngvRukLQXwG2', // Helius
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
        operator: '2sHNuid4rus4sK2EmndLeZcPNKkgzuEoc8Vro3PH2qop', // Hashkey
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
        operator: '5TGRFaLy3eF93pSNiPamCgvZUN3gzdYcs7jA3iCAsd1L', // InfStones
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
        operator: 'EkroMQiZJfphVd9iPvR4zMCHasTW72Uh1mFYkTxtQuY6', // StakingFacilities
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
        operator: '574DmorRvpaYrSrBRUwAjG7bBmrZYiTW3Fc8mvQatFqo', // Adrastea
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
        operator: 'C6AF8qGCo2dL815ziRCmfdbFeL5xbRLuSTSZzTGBH68y', // Figment
      }),
    () =>
      ctx.fund.addRestakingVaultDelegation.execute({
        vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
        operator: '6AxtdRGAaiAyqcwxVBHsH3xtqCbQuffaiE4epT4koTxk', // Staked
      }),
    () =>
      ctx.fund.updateRestakingVaultStrategy.executeChained({
        vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
        delegations: [
          {
            operator: '574DmorRvpaYrSrBRUwAjG7bBmrZYiTW3Fc8mvQatFqo',
            tokenAllocationWeight: 92n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: '7yofWXChEHkPTSnyFdKx2Smq5iWVbGB4P1dkdC6zHWYR',
            tokenAllocationWeight: 1n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: 'FzZ9EXmHv7ANCXijpALUBzCza6wYNprnsfaEHuoNx9sE',
            tokenAllocationWeight: 92n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: 'C6AF8qGCo2dL815ziRCmfdbFeL5xbRLuSTSZzTGBH68y',
            tokenAllocationWeight: 0n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: '2sHNuid4rus4sK2EmndLeZcPNKkgzuEoc8Vro3PH2qop',
            tokenAllocationWeight: 0n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: 'BFEsrxFPsBcY2hR5kgyfKnpwgEc8wYQdngvRukLQXwG2',
            tokenAllocationWeight: 4n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: '5TGRFaLy3eF93pSNiPamCgvZUN3gzdYcs7jA3iCAsd1L',
            tokenAllocationWeight: 92n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: '29rxXT5zbTR1ctiooHtb1Sa1TD4odzhQHsrLz3D78G5w',
            tokenAllocationWeight: 0n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: 'LKFpfXtBkH5b7D9mo8dPcjCLZCZpmLQC9ELkbkyVdah',
            tokenAllocationWeight: 92n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: 'GZxp4e2Tm3Pw9GyAaxuF6odT3XkRM96jpZkp3nxhoK4Y',
            tokenAllocationWeight: 2n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: '6AxtdRGAaiAyqcwxVBHsH3xtqCbQuffaiE4epT4koTxk',
            tokenAllocationWeight: 0n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: 'EkroMQiZJfphVd9iPvR4zMCHasTW72Uh1mFYkTxtQuY6',
            tokenAllocationWeight: 0n,
            tokenAllocationCapacityAmount: MAX_U64,
          },
          {
            operator: 'CA8PaNSoFWzvbCJ2oK3QxBEutgyHSTT5omEptpj8YHPY',
            tokenAllocationWeight: 3n,
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
