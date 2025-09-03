import * as token from '@solana-program/token';
import {
  Address,
  createNoopSigner,
  getAddressEncoder,
  getBytesEncoder,
  getProgramDerivedAddress,
} from '@solana/kit';
import * as v from 'valibot';
import * as jitoVault from '../../../clients/js/fragmetric-sdk/src/generated/jito_vault';
import { getRestakingAnchorEventDecoders } from '../../../clients/js/fragmetric-sdk/src/programs/restaking/events';
import type { TestSuiteContext } from '../../testutil';

export async function initializeFragX(
  testCtx: TestSuiteContext,
  index: number
) {
  const { validator, restaking, sdk } = testCtx;
  const { MAX_U64 } = sdk;

  const fragXSigner = await validator.getSigner('fragX' + index);
  const fragXWrappedReceiptTokenSigner = await validator.getSigner(
    'fragXWrappedReceiptToken' + index
  );

  const ctx = restaking.receiptTokenMint(fragXSigner as any);

  const initializationTasks = [
    // initialize receipt token mint and transfer hook metadata
    () =>
      ctx.initializeMint.execute(
        {
          name: 'Fragmetric Restaked SOL',
          symbol: 'fragSOL',
          uri: 'https://quicknode.quicknode-ipfs.com/ipfs/QmcueajXkNzoYRhcCv323PMC8VVGiDvXaaVXkMyYcyUSRw',
          description: `fragSOL is Solana's first native LRT that provides optimized restaking rewards.`,
          decimals: 9,
        },
        {
          signers: [fragXSigner],
        }
      ),
    () => ctx.initializeOrUpdateExtraAccountMetaList.execute(null),

    // initialize fund account and configuration
    () =>
      ctx.fund.initializeOrUpdateAccount.executeChained({ targetVersion: 19 }),
    () =>
      ctx.fund.updateGeneralStrategy.execute({
        depositEnabled: true,
        donationEnabled: true,
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
        solDepositable: true,
        solAccumulatedDepositAmount: null,
        solAccumulatedDepositCapacityAmount: MAX_U64,
        solWithdrawable: true,
        solWithdrawalNormalReserveRateBps: 0,
        solWithdrawalNormalReserveMaxAmount: MAX_U64,
      }),

    // initialize address lookup table (1)
    () =>
      ctx.fund.addressLookupTable
        .resolveFrequentlyUsedAddresses()
        .then((addresses) =>
          ctx.fund.addressLookupTable.initializeOrUpdateAccount.executeChained({
            addresses,
          })
        ),
    // wait for two slots to activate ALT (1)
    () => validator.skipSlots(2n),

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
        tokenDepositable: true,
        tokenAccumulatedDepositAmount: null,
        tokenAccumulatedDepositCapacityAmount: MAX_U64,
        tokenWithdrawable: false,
        tokenWithdrawalNormalReserveRateBps: 0,
        tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
        solAllocationWeight: 1n,
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
        tokenDepositable: true,
        tokenAccumulatedDepositAmount: null,
        tokenAccumulatedDepositCapacityAmount: MAX_U64,
        tokenWithdrawable: false,
        tokenWithdrawalNormalReserveRateBps: 0,
        tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
        solAllocationWeight: 1n,
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
        solAllocationWeight: 1n,
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
        tokenDepositable: true,
        tokenAccumulatedDepositAmount: null,
        tokenAccumulatedDepositCapacityAmount: MAX_U64,
        tokenWithdrawable: false,
        tokenWithdrawalNormalReserveRateBps: 0,
        tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
        solAllocationWeight: 1n,
        solAllocationCapacityAmount: MAX_U64,
      }),
    () =>
      ctx.fund.addSupportedToken.execute({
        mint: 'bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1',
        pricingSource: {
          __kind: 'SPLStakePool',
          address: 'stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi',
        },
      }),
    () =>
      ctx.fund.updateAssetStrategy.execute({
        tokenMint: 'bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1',
        tokenDepositable: true,
        tokenAccumulatedDepositAmount: null,
        tokenAccumulatedDepositCapacityAmount: MAX_U64,
        tokenWithdrawable: false,
        tokenWithdrawalNormalReserveRateBps: 0,
        tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
        solAllocationWeight: 1n,
        solAllocationCapacityAmount: MAX_U64,
      }),
    () =>
      ctx.fund.addSupportedToken.execute({
        mint: 'Dso1bDeDjCQxTrWHqUUi63oBvV7Mdm6WaobLbQ7gnPQ',
        pricingSource: {
          __kind: 'SanctumSingleValidatorSPLStakePool',
          address: '9mhGNSPArRMHpLDMSmxAvuoizBqtBGqYdT8WGuqgxNdn',
        },
      }),
    () =>
      ctx.fund.updateAssetStrategy.execute({
        tokenMint: 'Dso1bDeDjCQxTrWHqUUi63oBvV7Mdm6WaobLbQ7gnPQ',
        tokenDepositable: true,
        tokenAccumulatedDepositAmount: null,
        tokenAccumulatedDepositCapacityAmount: MAX_U64,
        tokenWithdrawable: false,
        tokenWithdrawalNormalReserveRateBps: 0,
        tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
        solAllocationWeight: 1n,
        solAllocationCapacityAmount: MAX_U64,
      }),
    () =>
      ctx.fund.addSupportedToken.execute({
        mint: 'FRAGME9aN7qzxkHPmVP22tDhG87srsR9pr5SY9XdRd9R',
        pricingSource: {
          __kind: 'SanctumSingleValidatorSPLStakePool',
          address: 'LUKAypUYCVCptMKuN7ug3NGyRFz6p3SvKLHEXudS56X',
        },
      }),
    () =>
      ctx.fund.updateAssetStrategy.execute({
        tokenMint: 'FRAGME9aN7qzxkHPmVP22tDhG87srsR9pr5SY9XdRd9R',
        tokenDepositable: true,
        tokenAccumulatedDepositAmount: null,
        tokenAccumulatedDepositCapacityAmount: MAX_U64,
        tokenWithdrawable: false,
        tokenWithdrawalNormalReserveRateBps: 0,
        tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
        solAllocationWeight: 1n,
        solAllocationCapacityAmount: MAX_U64,
      }),

    // () =>
    //   ctx.fund.addSupportedToken.execute({
    //     mint: 'he1iusmfkpAdwvxLNGV8Y1iSbj4rUy6yMhEA3fotn9A',
    //     pricingSource: {
    //       __kind: 'SanctumSingleValidatorSPLStakePool',
    //       address: '3wK2g8ZdzAH8FJ7PKr2RcvGh7V9VYson5hrVsJM5Lmws',
    //     },
    //   }),
    // () =>
    //   ctx.fund.updateAssetStrategy.execute({
    //     tokenMint: 'he1iusmfkpAdwvxLNGV8Y1iSbj4rUy6yMhEA3fotn9A',
    //     tokenDepositable: true,
    //     tokenAccumulatedDepositAmount: null,
    //     tokenAccumulatedDepositCapacityAmount: MAX_U64,
    //     tokenWithdrawable: false,
    //     tokenWithdrawalNormalReserveRateBps: 0,
    //     tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
    //     solAllocationWeight: 1n,
    //     solAllocationCapacityAmount: MAX_U64,
    //   }),
    // () =>
    //   ctx.fund.addSupportedToken.execute({
    //     mint: 'roxDFxTFHufJBFy3PgzZcgz6kwkQNPZpi9RfpcAv4bu',
    //     pricingSource: {
    //       __kind: 'SanctumSingleValidatorSPLStakePool',
    //       address: 'BuMRVW5uUQqJmguCk4toGh7DB3CcJt6dk64JiUMdYS22',
    //     },
    //   }),
    // () =>
    //   ctx.fund.updateAssetStrategy.execute({
    //     tokenMint: 'roxDFxTFHufJBFy3PgzZcgz6kwkQNPZpi9RfpcAv4bu',
    //     tokenDepositable: true,
    //     tokenAccumulatedDepositAmount: null,
    //     tokenAccumulatedDepositCapacityAmount: MAX_U64,
    //     tokenWithdrawable: false,
    //     tokenWithdrawalNormalReserveRateBps: 0,
    //     tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
    //     solAllocationWeight: 1n,
    //     solAllocationCapacityAmount: MAX_U64,
    //   }),
    // () =>
    //   ctx.fund.addSupportedToken.execute({
    //     mint: 'sctmadV2fcLtrxjzYhTZzwAGjXUXKtYSBrrM36EtdcY',
    //     pricingSource: {
    //       __kind: 'SanctumSingleValidatorSPLStakePool',
    //       address: '8iax3u8PEcP6VhBtLLG7QAoSrCp7fUbCJtmHPrqHxdas',
    //     },
    //   }),
    // () =>
    //   ctx.fund.updateAssetStrategy.execute({
    //     tokenMint: 'sctmadV2fcLtrxjzYhTZzwAGjXUXKtYSBrrM36EtdcY',
    //     tokenDepositable: true,
    //     tokenAccumulatedDepositAmount: null,
    //     tokenAccumulatedDepositCapacityAmount: MAX_U64,
    //     tokenWithdrawable: false,
    //     tokenWithdrawalNormalReserveRateBps: 0,
    //     tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
    //     solAllocationWeight: 1n,
    //     solAllocationCapacityAmount: MAX_U64,
    //   }),
    // () =>
    //   ctx.fund.addSupportedToken.execute({
    //     mint: 'BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs',
    //     pricingSource: {
    //       __kind: 'SanctumSingleValidatorSPLStakePool',
    //       address: 'ArAQfbzsdotoKB5jJcZa3ajQrrPcWr2YQoDAEAiFxJAC',
    //     },
    //   }),
    // () =>
    //   ctx.fund.updateAssetStrategy.execute({
    //     tokenMint: 'BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs',
    //     tokenDepositable: true,
    //     tokenAccumulatedDepositAmount: null,
    //     tokenAccumulatedDepositCapacityAmount: MAX_U64,
    //     tokenWithdrawable: false,
    //     tokenWithdrawalNormalReserveRateBps: 0,
    //     tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
    //     solAllocationWeight: 1n,
    //     solAllocationCapacityAmount: MAX_U64,
    //   }),
    // () =>
    //   ctx.fund.addSupportedToken.execute({
    //     mint: 'vSoLxydx6akxyMD9XEcPvGYNGq6Nn66oqVb3UkGkei7',
    //     pricingSource: {
    //       __kind: 'SPLStakePool',
    //       address: 'Fu9BYC6tWBo1KMKaP3CFoKfRhqv9akmy3DuYwnCyWiyC',
    //     },
    //   }),
    // () =>
    //   ctx.fund.updateAssetStrategy.execute({
    //     tokenMint: 'vSoLxydx6akxyMD9XEcPvGYNGq6Nn66oqVb3UkGkei7',
    //     tokenDepositable: true,
    //     tokenAccumulatedDepositAmount: null,
    //     tokenAccumulatedDepositCapacityAmount: MAX_U64,
    //     tokenWithdrawable: false,
    //     tokenWithdrawalNormalReserveRateBps: 0,
    //     tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
    //     solAllocationWeight: 1n,
    //     solAllocationCapacityAmount: MAX_U64,
    //   }),
    // () =>
    //   ctx.fund.addSupportedToken.execute({
    //     mint: 'HUBsveNpjo5pWqNkH57QzxjQASdTVXcSK7bVKTSZtcSX',
    //     pricingSource: {
    //       __kind: 'SanctumSingleValidatorSPLStakePool',
    //       address: 'ECRqn7gaNASuvTyC5xfCUjehWZCSowMXstZiM5DNweyB',
    //     },
    //   }),
    // () =>
    //   ctx.fund.updateAssetStrategy.execute({
    //     tokenMint: 'HUBsveNpjo5pWqNkH57QzxjQASdTVXcSK7bVKTSZtcSX',
    //     tokenDepositable: true,
    //     tokenAccumulatedDepositAmount: null,
    //     tokenAccumulatedDepositCapacityAmount: MAX_U64,
    //     tokenWithdrawable: false,
    //     tokenWithdrawalNormalReserveRateBps: 0,
    //     tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
    //     solAllocationWeight: 1n,
    //     solAllocationCapacityAmount: MAX_U64,
    //   }),
    // () =>
    //   ctx.fund.addSupportedToken.execute({
    //     mint: 'picobAEvs6w7QEknPce34wAE4gknZA9v5tTonnmHYdX',
    //     pricingSource: {
    //       __kind: 'SanctumSingleValidatorSPLStakePool',
    //       address: '8Dv3hNYcEWEaa4qVx9BTN1Wfvtha1z8cWDUXb7KVACVe',
    //     },
    //   }),
    // () =>
    //   ctx.fund.updateAssetStrategy.execute({
    //     tokenMint: 'picobAEvs6w7QEknPce34wAE4gknZA9v5tTonnmHYdX',
    //     tokenDepositable: true,
    //     tokenAccumulatedDepositAmount: null,
    //     tokenAccumulatedDepositCapacityAmount: MAX_U64,
    //     tokenWithdrawable: false,
    //     tokenWithdrawalNormalReserveRateBps: 0,
    //     tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
    //     solAllocationWeight: 1n,
    //     solAllocationCapacityAmount: MAX_U64,
    //   }),
    // () =>
    //   ctx.fund.addSupportedToken.execute({
    //     mint: 'strng7mqqc1MBJJV6vMzYbEqnwVGvKKGKedeCvtktWA',
    //     pricingSource: {
    //       __kind: 'SanctumSingleValidatorSPLStakePool',
    //       address: 'GZDX5JYXDzCEDL3kybhjN7PSixL4ams3M2G4CvWmMmm5',
    //     },
    //   }),
    // () =>
    //   ctx.fund.updateAssetStrategy.execute({
    //     tokenMint: 'strng7mqqc1MBJJV6vMzYbEqnwVGvKKGKedeCvtktWA',
    //     tokenDepositable: true,
    //     tokenAccumulatedDepositAmount: null,
    //     tokenAccumulatedDepositCapacityAmount: MAX_U64,
    //     tokenWithdrawable: false,
    //     tokenWithdrawalNormalReserveRateBps: 0,
    //     tokenWithdrawalNormalReserveMaxAmount: MAX_U64,
    //     solAllocationWeight: 1n,
    //     solAllocationCapacityAmount: MAX_U64,
    //   }),

    // not needed at the real fragsol, but it's included for unit test
    () =>
      ctx.fund.addTokenSwapStrategy.execute({
        fromTokenMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
        toTokenMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
        swapSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: 'G2FiE1yn9N9ZJx5e1E2LxxMnHvb1H3hCuHLPfKJ98smA',
        },
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

    ...(index == 0
      ? [
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
              // mint: fragXNormalizedTokenSigner.address,
              mint: '4noNmx2RpxK4zdr68Fq1CYM5VhN4yjgGZEFyuB7t2pBX',
            }),
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
          () =>
            ctx.normalizedTokenPool.addSupportedToken.execute({
              mint: 'bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1',
              pricingSource: {
                __kind: 'SPLStakePool',
                address: 'stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi',
              },
            }),
          () =>
            ctx.normalizedTokenPool.addSupportedToken.execute({
              mint: 'Dso1bDeDjCQxTrWHqUUi63oBvV7Mdm6WaobLbQ7gnPQ',
              pricingSource: {
                __kind: 'SanctumSingleValidatorSPLStakePool',
                address: '9mhGNSPArRMHpLDMSmxAvuoizBqtBGqYdT8WGuqgxNdn',
              },
            }),
          () =>
            ctx.normalizedTokenPool.addSupportedToken.execute({
              mint: 'FRAGME9aN7qzxkHPmVP22tDhG87srsR9pr5SY9XdRd9R',
              pricingSource: {
                __kind: 'SanctumSingleValidatorSPLStakePool',
                address: 'LUKAypUYCVCptMKuN7ug3NGyRFz6p3SvKLHEXudS56X',
              },
            }),
        ]
      : [
          // initialize fund vault supported token account
          () => {
            const initializeFundNormalizedToken =
              new sdk.TransactionTemplateContext(
                ctx.fund,
                v.object({
                  mint: v.string(),
                }),
                {
                  description:
                    'initialize normalized token pool account and enable',
                  anchorEventDecoders: getRestakingAnchorEventDecoders(
                    'fundManagerUpdatedFund'
                  ),
                  addressLookupTables: [
                    () => ctx.fund.addressLookupTable.resolveAddress(true),
                  ],
                  instructions: [
                    async (parent, args, overrides) => {
                      ctx.fund.addressLookupTable;
                      const [
                        existingNormalizedTokenMint,
                        data,
                        fundReserve,
                        payer,
                      ] = await Promise.all([
                        parent.parent.normalizedTokenMint.resolveAddress(true),
                        parent.parent.resolve(true),
                        parent.parent.fund.reserve.resolveAddress(),
                        sdk.transformAddressResolverVariant(
                          overrides.feePayer ??
                            ctx.runtime.options.transaction.feePayer ??
                            (() => Promise.resolve(null))
                        )(parent),
                      ]);
                      if (
                        !(!existingNormalizedTokenMint && data && fundReserve)
                      )
                        throw new Error('invalid context');
                      const admin = restaking.knownAddresses.admin;
                      const fundManager = restaking.knownAddresses.fundManager;

                      return Promise.all([
                        token.getCreateAssociatedTokenIdempotentInstructionAsync(
                          {
                            payer: createNoopSigner(payer as Address),
                            mint: args.mint as Address,
                            owner: fundReserve,
                            tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
                          }
                        ),
                        sdk.restakingTypes
                          .getFundManagerInitializeFundNormalizedTokenInstructionAsync(
                            {
                              fundManager: createNoopSigner(
                                fundManager as Address<string>
                              ),
                              receiptTokenMint: data.receiptTokenMint,
                              program: restaking.address,
                              normalizedTokenMint: args.mint as Address,
                            },
                            {
                              programAddress: restaking.address,
                            }
                          )
                          .then((ix) => {
                            // add pricing sources
                            for (const accountMeta of data.__pricingSources) {
                              ix.accounts.push(accountMeta);
                            }
                            ix.accounts.push(ix.accounts[7]); // ntp
                            return ix;
                          }),
                      ]);
                    },
                  ],
                }
              );

            return initializeFundNormalizedToken.execute({
              mint: '4noNmx2RpxK4zdr68Fq1CYM5VhN4yjgGZEFyuB7t2pBX',
            });
          },
        ]),
    // () =>
    //   ctx.normalizedTokenPool.addSupportedToken.execute({
    //     mint: 'he1iusmfkpAdwvxLNGV8Y1iSbj4rUy6yMhEA3fotn9A',
    //     pricingSource: {
    //       __kind: 'SanctumSingleValidatorSPLStakePool',
    //       address: '3wK2g8ZdzAH8FJ7PKr2RcvGh7V9VYson5hrVsJM5Lmws',
    //     },
    //   }),
    // () =>
    //   ctx.normalizedTokenPool.addSupportedToken.execute({
    //     mint: 'roxDFxTFHufJBFy3PgzZcgz6kwkQNPZpi9RfpcAv4bu',
    //     pricingSource: {
    //       __kind: 'SanctumSingleValidatorSPLStakePool',
    //       address: 'BuMRVW5uUQqJmguCk4toGh7DB3CcJt6dk64JiUMdYS22',
    //     },
    //   }),
    // () =>
    //   ctx.normalizedTokenPool.addSupportedToken.execute({
    //     mint: 'sctmadV2fcLtrxjzYhTZzwAGjXUXKtYSBrrM36EtdcY',
    //     pricingSource: {
    //       __kind: 'SanctumSingleValidatorSPLStakePool',
    //       address: '8iax3u8PEcP6VhBtLLG7QAoSrCp7fUbCJtmHPrqHxdas',
    //     },
    //   }),

    // () =>
    //   ctx.normalizedTokenPool.addSupportedToken.execute({
    //     mint: 'BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs',
    //     pricingSource: {
    //       __kind: 'SanctumSingleValidatorSPLStakePool',
    //       address: 'ArAQfbzsdotoKB5jJcZa3ajQrrPcWr2YQoDAEAiFxJAC',
    //     },
    //   }),
    // () =>
    //   ctx.normalizedTokenPool.addSupportedToken.execute({
    //     mint: 'vSoLxydx6akxyMD9XEcPvGYNGq6Nn66oqVb3UkGkei7',
    //     pricingSource: {
    //       __kind: 'SPLStakePool',
    //       address: 'Fu9BYC6tWBo1KMKaP3CFoKfRhqv9akmy3DuYwnCyWiyC',
    //     },
    //   }),
    // () =>
    //   ctx.normalizedTokenPool.addSupportedToken.execute({
    //     mint: 'HUBsveNpjo5pWqNkH57QzxjQASdTVXcSK7bVKTSZtcSX',
    //     pricingSource: {
    //       __kind: 'SanctumSingleValidatorSPLStakePool',
    //       address: 'ECRqn7gaNASuvTyC5xfCUjehWZCSowMXstZiM5DNweyB',
    //     },
    //   }),
    // () =>
    //   ctx.normalizedTokenPool.addSupportedToken.execute({
    //     mint: 'picobAEvs6w7QEknPce34wAE4gknZA9v5tTonnmHYdX',
    //     pricingSource: {
    //       __kind: 'SanctumSingleValidatorSPLStakePool',
    //       address: '8Dv3hNYcEWEaa4qVx9BTN1Wfvtha1z8cWDUXb7KVACVe',
    //     },
    //   }),
    // () =>
    //   ctx.normalizedTokenPool.addSupportedToken.execute({
    //     mint: 'strng7mqqc1MBJJV6vMzYbEqnwVGvKKGKedeCvtktWA',
    //     pricingSource: {
    //       __kind: 'SanctumSingleValidatorSPLStakePool',
    //       address: 'GZDX5JYXDzCEDL3kybhjN7PSixL4ams3M2G4CvWmMmm5',
    //     },
    //   }),

    // initialize address lookup table (2)
    () =>
      ctx.fund.addressLookupTable
        .resolveFrequentlyUsedAddresses()
        .then((addresses) =>
          ctx.fund.addressLookupTable.initializeOrUpdateAccount.executeChained({
            addresses,
          })
        ),
    // wait for two slots to activate ALT (2)
    () => validator.skipSlots(2n),

    // initialize wrapped token mint and configuration
    () =>
      ctx.wrappedTokenMint.initializeMint.execute(
        {
          mint: fragXWrappedReceiptTokenSigner.address,
          name: 'Wrapped Fragmetric Restaked SOL',
          symbol: 'wfragSOL',
          uri: 'https://quicknode.quicknode-ipfs.com/ipfs/QmaTVVmyvbJXs2Rqcqs76N5UiuPZ2iKCKrb5BpyB13vwzU',
          description: `wfragSOL is Solana's SPL token version of fragSOL that maximizes DeFi support and composability.`,
          decimals: 9,
        },
        {
          signers: [fragXWrappedReceiptTokenSigner],
        }
      ),
    () =>
      ctx.fund.initializeWrappedToken.execute({
        mint: fragXWrappedReceiptTokenSigner.address,
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
    ...(index === 0
      ? [
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
        ]
      : (() => {
          const addFundRestakingVaultDelegation =
            new sdk.TransactionTemplateContext(
              ctx.fund,
              v.object({
                vault: v.string(),
                operator: v.string(),
              }),
              {
                description:
                  'add a new operator delegation to a restaking vault',
                anchorEventDecoders: getRestakingAnchorEventDecoders(
                  'fundManagerUpdatedFund'
                ),
                addressLookupTables: [
                  () => ctx.fund.addressLookupTable.resolveAddress(true),
                ],
                instructions: [
                  async (parent, args, overrides) => {
                    const [data, vaultStrategies, payer] = await Promise.all([
                      parent.parent.resolve(true),
                      parent.resolveRestakingVaultStrategies(true),
                      sdk.transformAddressResolverVariant(
                        overrides.feePayer ??
                          ctx.runtime.options.transaction.feePayer ??
                          (() => Promise.resolve(null))
                      )(parent),
                    ]);
                    const vaultStrategy = vaultStrategies?.find(
                      (item) => item.vault == args.vault
                    );
                    if (!(data && vaultStrategy))
                      throw new Error('invalid context');
                    const fundManager = restaking.knownAddresses.fundManager;

                    if (
                      vaultStrategy.pricingSource.__kind == 'JitoRestakingVault'
                    ) {
                      const [[vaultOperatorDelegation]] = await Promise.all([
                        getProgramDerivedAddress({
                          programAddress: jitoVault.JITO_VAULT_PROGRAM_ADDRESS,
                          seeds: [
                            getBytesEncoder().encode(
                              Buffer.from('vault_operator_delegation')
                            ),
                            getAddressEncoder().encode(args.vault as Address),
                            getAddressEncoder().encode(
                              args.operator as Address
                            ),
                          ],
                        }),
                      ]);

                      return Promise.all([
                        sdk.restakingTypes.getFundManagerInitializeFundRestakingVaultDelegationInstructionAsync(
                          {
                            vaultOperatorDelegation,
                            vaultAccount: args.vault as Address,
                            operatorAccount: args.operator as Address,
                            fundManager: createNoopSigner(fundManager),
                            program: restaking.address,
                            receiptTokenMint: data.receiptTokenMint,
                          },
                          {
                            programAddress: restaking.address,
                          }
                        ),
                      ]);
                    }

                    throw new Error(
                      'unsupported restaking vault pricing source'
                    );
                  },
                ],
              }
            );

          return [
            () =>
              addFundRestakingVaultDelegation.execute({
                vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
                operator: 'FzZ9EXmHv7ANCXijpALUBzCza6wYNprnsfaEHuoNx9sE', // Everstake
              }),
            () =>
              addFundRestakingVaultDelegation.execute({
                vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
                operator: '29rxXT5zbTR1ctiooHtb1Sa1TD4odzhQHsrLz3D78G5w', // KILN
              }),
            () =>
              addFundRestakingVaultDelegation.execute({
                vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
                operator: 'LKFpfXtBkH5b7D9mo8dPcjCLZCZpmLQC9ELkbkyVdah', // Luganodes
              }),
            () =>
              addFundRestakingVaultDelegation.execute({
                vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
                operator: 'GZxp4e2Tm3Pw9GyAaxuF6odT3XkRM96jpZkp3nxhoK4Y', // PierTwo
              }),
            () =>
              addFundRestakingVaultDelegation.execute({
                vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
                operator: 'CA8PaNSoFWzvbCJ2oK3QxBEutgyHSTT5omEptpj8YHPY', // Temporal
              }),
            () =>
              addFundRestakingVaultDelegation.execute({
                vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
                operator: '7yofWXChEHkPTSnyFdKx2Smq5iWVbGB4P1dkdC6zHWYR', // ChorusOne
              }),
            () =>
              addFundRestakingVaultDelegation.execute({
                vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
                operator: 'BFEsrxFPsBcY2hR5kgyfKnpwgEc8wYQdngvRukLQXwG2', // Helius
              }),
            () =>
              addFundRestakingVaultDelegation.execute({
                vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
                operator: '2sHNuid4rus4sK2EmndLeZcPNKkgzuEoc8Vro3PH2qop', // Hashkey
              }),
            () =>
              addFundRestakingVaultDelegation.execute({
                vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
                operator: '5TGRFaLy3eF93pSNiPamCgvZUN3gzdYcs7jA3iCAsd1L', // InfStones
              }),
            () =>
              addFundRestakingVaultDelegation.execute({
                vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
                operator: 'EkroMQiZJfphVd9iPvR4zMCHasTW72Uh1mFYkTxtQuY6', // StakingFacilities
              }),
            () =>
              addFundRestakingVaultDelegation.execute({
                vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
                operator: '574DmorRvpaYrSrBRUwAjG7bBmrZYiTW3Fc8mvQatFqo', // Adrastea
              }),
            () =>
              addFundRestakingVaultDelegation.execute({
                vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
                operator: 'C6AF8qGCo2dL815ziRCmfdbFeL5xbRLuSTSZzTGBH68y', // Figment
              }),
            () =>
              addFundRestakingVaultDelegation.execute({
                vault: 'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S',
                operator: '6AxtdRGAaiAyqcwxVBHsH3xtqCbQuffaiE4epT4koTxk', // Staked
              }),
          ];
        })()),
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

    // initialize address lookup table (3)
    () =>
      ctx.fund.addressLookupTable
        .resolveFrequentlyUsedAddresses()
        .then((addresses) =>
          ctx.fund.addressLookupTable.initializeOrUpdateAccount.executeChained({
            addresses,
          })
        ),
    // wait for two slots to activate ALT (3)
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

  return { ...testCtx, initializationTasks, ctx };
}
