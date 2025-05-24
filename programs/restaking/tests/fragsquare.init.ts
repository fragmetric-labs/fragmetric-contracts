import {
  Address,
  getAddressEncoder,
  getBytesEncoder,
  getProgramDerivedAddress,
} from '@solana/kit';
import { VirtualVaultAccountContext } from '../../../clients/js/fragmetric-sdk/src/programs/restaking/virtual_vault';
import { TestSuiteContext } from '../../testutil';

export async function initializeFragSquare(testCtx: TestSuiteContext) {
  const { validator, restaking, sdk } = testCtx;
  const { MAX_U64 } = sdk;

  const ctx = restaking.fragSquare;
  const [vaultAddress] = await getProgramDerivedAddress({
    programAddress: restaking.program.address as unknown as Address,
    seeds: [
      getBytesEncoder().encode(Buffer.from('virtual_vault')),
      getAddressEncoder().encode(
        '8vEunBQvD3L4aNnRPyQzfQ7pecq4tPb46PjZVKUnTP9i' as Address
      ), // vrt
    ],
  });

  const initializationTasks = [
    // initialize receipt token mint and transfer hook metadata
    () =>
      ctx.initializeMint.execute({
        name: 'Fragmetric Restaked FRAG',
        symbol: 'fragSquare',
        uri: '',
        description: 'Fragmetric Restaked FRAG',
        decimals: 9,
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
    () =>
      ctx.reward.addReward.execute({
        mint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
        decimals: 9,
        name: 'JitoSOL',
        description: 'JitoSOL insentive',
      }),
    () =>
      ctx.reward.settleReward.execute({
        mint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
        amount: 0n,
      }),
    () =>
      ctx.reward.addReward.execute({
        mint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
        decimals: 6,
        name: 'ZEUS',
        description: 'ZEUS insentive',
      }),
    () =>
      ctx.reward.updateReward.execute({
        mint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
        claimable: true,
      }),
    () => validator.skipSlots(1n),
    () =>
      ctx.reward.settleReward.execute({
        mint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
        amount: 0n,
      }),

    // initialize virtual vault
    async () => {
      const vaultContext = ctx.fund.restakingVault(
        vaultAddress,
        '11111111111111111111111111111111'
      );
      if (!(vaultContext && vaultContext instanceof VirtualVaultAccountContext))
        throw new Error('invalid context: virtual vault not found');

      return vaultContext.initializeVrtMint.execute({
        mint: '8vEunBQvD3L4aNnRPyQzfQ7pecq4tPb46PjZVKUnTP9i',
        name: 'fragSquare Virtual Vault Receipt Token Mint',
        symbol: 'fragSquareVVrt',
        uri: '',
        description: 'fragSquare Virtual Vault Receipt Token Mint',
        decimals: 9,
      });
    },
    () =>
      ctx.fund.addRestakingVault.execute({
        vault: vaultAddress,
        pricingSource: {
          __kind: 'VirtualVault',
          address: vaultAddress,
        },
        vstMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
        vrtMint: '8vEunBQvD3L4aNnRPyQzfQ7pecq4tPb46PjZVKUnTP9i',
      }),

    // configure reward settings
    () =>
      ctx.fund.addRestakingVaultCompoundingReward.execute({
        vault: vaultAddress,
        rewardTokenMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
      }),
    () =>
      ctx.fund.addRestakingVaultDistributingReward.execute({
        vault: vaultAddress,
        rewardTokenMint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
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
