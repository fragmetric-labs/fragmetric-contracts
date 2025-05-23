import {
  Address,
  getAddressEncoder,
  getBytesEncoder,
  getProgramDerivedAddress,
} from '@solana/kit';
import { afterAll, beforeAll, describe, expect, test } from 'vitest';
import { createTestSuiteContext, expectMasked } from '../../testutil';
import { initializeFragSquare } from './fragsquare.init';

describe('restaking.fragSquare test', async () => {
  const testCtx = await initializeFragSquare(await createTestSuiteContext());

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  const { validator, feePayer, restaking, initializationTasks } = testCtx;
  const ctx = restaking.fragSquare;

  const [signer1, signer2] = await Promise.all([
    validator
      .newSigner('fragSquareDepositTestSigner1', 100_000_000_000n)
      .then(async (signer) => {
        await Promise.all([
          validator.airdropToken(
            signer.address,
            'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
            100_000_000_000n
          ),
        ]);
        return signer;
      }),
    validator
      .newSigner('fragSquareDepositTestSigner2', 100_000_000_000n)
      .then(async (signer) => {
        await Promise.all([
          validator.airdropToken(
            signer.address,
            'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
            100_000_000_000n
          ),
        ]);
        return signer;
      }),
  ]);
  const user1 = ctx.user(signer1);
  const user2 = ctx.user(signer2);

  /** 1. configuration */
  test('restaking.fragSquare initializationTasks snapshot', async () => {
    await expectMasked(initializationTasks).resolves.toMatchSnapshot();
  });

  test('restaking.fragSquare.resolve', async () => {
    await expect(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "G45gQa12Uwvnrp2Yb9oWTSwZSEHZWL71QDWvyLz23bNc",
        "__pricingSources": [
          {
            "address": "Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb",
            "role": 0,
          },
          {
            "address": "ENwxFTsCAbWyjGLHnLYv7JtDs8uvkhvdmRoKCAS7SEpk",
            "role": 0,
          },
        ],
        "depositResidualMicroReceiptTokenAmount": 0n,
        "metadata": null,
        "normalizedToken": null,
        "oneReceiptTokenAsSOL": 0n,
        "receiptTokenDecimals": 9,
        "receiptTokenMint": "DCoj5m7joWjP9T3iPH22q7bDBoGkgUX4ffoL1eQZstwk",
        "receiptTokenSupply": 0n,
        "restakingVaultReceiptTokens": [
          {
            "mint": "8vEunBQvD3L4aNnRPyQzfQ7pecq4tPb46PjZVKUnTP9i",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "vault": "ENwxFTsCAbWyjGLHnLYv7JtDs8uvkhvdmRoKCAS7SEpk",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 9,
            "depositable": true,
            "mint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1160715954n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": 1970-01-01T00:00:00.000Z,
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": null,
      }
    `);
  });

  test('restaking.fragSquare.fund.resolve', async () => {
    await expect(ctx.fund.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "assetStrategies": [
          {
            "solAccumulatedDepositAmount": 0n,
            "solAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "solDepositable": false,
            "solWithdrawable": false,
            "solWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "solWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 0n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": true,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
        ],
        "generalStrategy": {
          "depositEnabled": true,
          "donationEnabled": false,
          "transferEnabled": true,
          "withdrawalBatchThresholdSeconds": 1n,
          "withdrawalEnabled": true,
          "withdrawalFeeRateBps": 20,
        },
        "restakingVaultStrategies": [
          {
            "compoundingRewardTokenMints": [
              "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            ],
            "delegations": [],
            "distributingRewardTokens": [
              {
                "harvestThresholdIntervalSeconds": 0n,
                "harvestThresholdMaxAmount": 18446744073709551615n,
                "harvestThresholdMinAmount": 0n,
                "lastHarvestedAt": 0n,
                "mint": "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
              },
            ],
            "pricingSource": {
              "__kind": "VirtualVault",
              "address": "ENwxFTsCAbWyjGLHnLYv7JtDs8uvkhvdmRoKCAS7SEpk",
            },
            "solAllocationCapacityAmount": 0n,
            "solAllocationWeight": 0n,
            "vault": "ENwxFTsCAbWyjGLHnLYv7JtDs8uvkhvdmRoKCAS7SEpk",
          },
        ],
        "tokenSwapStrategies": [],
      }
    `);
  });

  test('restaking.fragSquare.reward.resolve', async () => {
    await expectMasked(ctx.reward.resolve(true)).resolves
      .toMatchInlineSnapshot(`
      {
        "basePool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "customContributionAccrualRateEnabled": false,
          "initialSlot": "MASKED(/[.*S|s]lots?$/)",
          "settlements": [
            {
              "blocks": [
                {
                  "amount": 0n,
                  "endingContribution": "MASKED(/[.*C|c]ontribution?$/)",
                  "endingSlot": "MASKED(/[.*S|s]lots?$/)",
                  "settledContribution": "MASKED(/[.*C|c]ontribution?$/)",
                  "settledSlots": "MASKED(/[.*S|s]lots?$/)",
                  "startingContribution": "MASKED(/[.*C|c]ontribution?$/)",
                  "startingSlot": "MASKED(/[.*S|s]lots?$/)",
                  "userSettledAmount": 0n,
                  "userSettledContribution": "MASKED(/[.*C|c]ontribution?$/)",
                },
              ],
              "claimedAmount": 0n,
              "claimedAmountUpdatedSlot": "MASKED(/[.*S|s]lots?$/)",
              "remainingAmount": 0n,
              "reward": {
                "claimable": false,
                "decimals": 9,
                "description": "JitoSOL insentive",
                "id": 1,
                "mint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
                "name": "JitoSOL",
                "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
              },
              "settledAmount": 0n,
              "settlementBlocksLastRewardPoolContribution": "MASKED(/[.*C|c]ontribution?$/)",
              "settlementBlocksLastSlot": "MASKED(/[.*S|s]lots?$/)",
            },
            {
              "blocks": [
                {
                  "amount": 0n,
                  "endingContribution": "MASKED(/[.*C|c]ontribution?$/)",
                  "endingSlot": "MASKED(/[.*S|s]lots?$/)",
                  "settledContribution": "MASKED(/[.*C|c]ontribution?$/)",
                  "settledSlots": "MASKED(/[.*S|s]lots?$/)",
                  "startingContribution": "MASKED(/[.*C|c]ontribution?$/)",
                  "startingSlot": "MASKED(/[.*S|s]lots?$/)",
                  "userSettledAmount": 0n,
                  "userSettledContribution": "MASKED(/[.*C|c]ontribution?$/)",
                },
              ],
              "claimedAmount": 0n,
              "claimedAmountUpdatedSlot": "MASKED(/[.*S|s]lots?$/)",
              "remainingAmount": 0n,
              "reward": {
                "claimable": true,
                "decimals": 6,
                "description": "ZEUS insentive",
                "id": 2,
                "mint": "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
                "name": "ZEUS",
                "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
              },
              "settledAmount": 0n,
              "settlementBlocksLastRewardPoolContribution": "MASKED(/[.*C|c]ontribution?$/)",
              "settlementBlocksLastSlot": "MASKED(/[.*S|s]lots?$/)",
            },
          ],
          "tokenAllocatedAmount": {
            "records": [],
            "totalAmount": 0n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "bonusPool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "customContributionAccrualRateEnabled": true,
          "initialSlot": "MASKED(/[.*S|s]lots?$/)",
          "settlements": [
            {
              "blocks": [
                {
                  "amount": 0n,
                  "endingContribution": "MASKED(/[.*C|c]ontribution?$/)",
                  "endingSlot": "MASKED(/[.*S|s]lots?$/)",
                  "settledContribution": "MASKED(/[.*C|c]ontribution?$/)",
                  "settledSlots": "MASKED(/[.*S|s]lots?$/)",
                  "startingContribution": "MASKED(/[.*C|c]ontribution?$/)",
                  "startingSlot": "MASKED(/[.*S|s]lots?$/)",
                  "userSettledAmount": 0n,
                  "userSettledContribution": "MASKED(/[.*C|c]ontribution?$/)",
                },
              ],
              "claimedAmount": 0n,
              "claimedAmountUpdatedSlot": "MASKED(/[.*S|s]lots?$/)",
              "remainingAmount": 0n,
              "reward": {
                "claimable": false,
                "decimals": 4,
                "description": "Airdrop point for fToken",
                "id": 0,
                "mint": "11111111111111111111111111111111",
                "name": "fPoint",
                "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
              },
              "settledAmount": 0n,
              "settlementBlocksLastRewardPoolContribution": "MASKED(/[.*C|c]ontribution?$/)",
              "settlementBlocksLastSlot": "MASKED(/[.*S|s]lots?$/)",
            },
          ],
          "tokenAllocatedAmount": {
            "records": [],
            "totalAmount": 0n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "receiptTokenMint": "DCoj5m7joWjP9T3iPH22q7bDBoGkgUX4ffoL1eQZstwk",
        "rewards": [
          {
            "claimable": false,
            "decimals": 4,
            "description": "Airdrop point for fToken",
            "id": 0,
            "mint": "11111111111111111111111111111111",
            "name": "fPoint",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          },
          {
            "claimable": false,
            "decimals": 9,
            "description": "JitoSOL insentive",
            "id": 1,
            "mint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "name": "JitoSOL",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          },
          {
            "claimable": true,
            "decimals": 6,
            "description": "ZEUS insentive",
            "id": 2,
            "mint": "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
            "name": "ZEUS",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          },
        ],
      }
    `);
  });

  /** 2. deposit */
  test('user can deposit JitoSOL', async () => {
    await expectMasked(
      user1.deposit.execute(
        {
          assetMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
          assetAmount: 5_000_000_000n,
        },
        { signers: [signer1] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": 5000000000n,
          "assetMint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
          "metadata": null,
        },
        "events": {
          "unknown": [],
          "userCreatedOrUpdatedFundAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "DCoj5m7joWjP9T3iPH22q7bDBoGkgUX4ffoL1eQZstwk",
            "user": "5zNZeAxc1YL7GUcFaMA1oXhzmCHVkQYCGxTRdF1SorxF",
            "userFundAccount": "FteL383dputRx9x5LpQSKKGoX9XJrSUgMTRRQ9TJW3AF",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "DCoj5m7joWjP9T3iPH22q7bDBoGkgUX4ffoL1eQZstwk",
            "user": "5zNZeAxc1YL7GUcFaMA1oXhzmCHVkQYCGxTRdF1SorxF",
            "userRewardAccount": "3Q6B1Hpp6FgqtcvLQUXCmdXEKq2tKhJbHDjNsJyYAW6q",
          },
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 5000000000n,
            "fundAccount": "4aQn7zN3sAYYbTjaJPRv9i9v2UipV2Lc49yYCW4Da5BZ",
            "mintedReceiptTokenAmount": 5000000000n,
            "receiptTokenMint": "DCoj5m7joWjP9T3iPH22q7bDBoGkgUX4ffoL1eQZstwk",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            },
            "updatedUserRewardAccounts": [
              "3Q6B1Hpp6FgqtcvLQUXCmdXEKq2tKhJbHDjNsJyYAW6q",
            ],
            "user": "5zNZeAxc1YL7GUcFaMA1oXhzmCHVkQYCGxTRdF1SorxF",
            "userFundAccount": "FteL383dputRx9x5LpQSKKGoX9XJrSUgMTRRQ9TJW3AF",
            "userReceiptTokenAccount": "3HnzMV1QvThCQoitcux2avpYVZBqE7XKah1h3USsT6sk",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "3XfdEQ5NkA9JZEfb6t9GLduGGSjLpVrQc4E4MsnUgkgA",
            },
            "walletProvider": {
              "__option": "None",
            },
          },
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);

    await expect(
      user1.receiptToken.resolve(true).then((res) => res?.amount)
    ).resolves.toEqual(5000000000n);

    await expect(user1.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "lamports": 99962596960n,
        "maxWithdrawalRequests": 4,
        "receiptTokenAmount": 5000000000n,
        "receiptTokenMint": "DCoj5m7joWjP9T3iPH22q7bDBoGkgUX4ffoL1eQZstwk",
        "supportedAssets": [
          {
            "amount": 95000000000n,
            "decimals": 9,
            "depositable": true,
            "mint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
          },
        ],
        "user": "5zNZeAxc1YL7GUcFaMA1oXhzmCHVkQYCGxTRdF1SorxF",
        "withdrawalRequests": [],
        "wrappedTokenAmount": 0n,
        "wrappedTokenMint": null,
      }
    `);

    await expect(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "G45gQa12Uwvnrp2Yb9oWTSwZSEHZWL71QDWvyLz23bNc",
        "__pricingSources": [
          {
            "address": "Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb",
            "role": 0,
          },
          {
            "address": "ENwxFTsCAbWyjGLHnLYv7JtDs8uvkhvdmRoKCAS7SEpk",
            "role": 0,
          },
        ],
        "depositResidualMicroReceiptTokenAmount": 0n,
        "metadata": null,
        "normalizedToken": null,
        "oneReceiptTokenAsSOL": 1160715954n,
        "receiptTokenDecimals": 9,
        "receiptTokenMint": "DCoj5m7joWjP9T3iPH22q7bDBoGkgUX4ffoL1eQZstwk",
        "receiptTokenSupply": 5000000000n,
        "restakingVaultReceiptTokens": [
          {
            "mint": "8vEunBQvD3L4aNnRPyQzfQ7pecq4tPb46PjZVKUnTP9i",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "vault": "ENwxFTsCAbWyjGLHnLYv7JtDs8uvkhvdmRoKCAS7SEpk",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 9,
            "depositable": true,
            "mint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "oneTokenAsReceiptToken": 1000000000n,
            "oneTokenAsSol": 1160715954n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 5000000000n,
            "operationTotalAmount": 5000000000n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 5000000000n,
            "withdrawalLastBatchProcessedAt": 1970-01-01T00:00:00.000Z,
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": null,
      }
    `);
  });

  /** 3. virtual vault harvest/compound */
  test('virtual vault harvest/compound', async () => {
    const [vaultAddress] = await getProgramDerivedAddress({
      programAddress: restaking.program.address as unknown as Address,
      seeds: [
        getBytesEncoder().encode(Buffer.from('virtual_vault')),
        getAddressEncoder().encode(
          '8vEunBQvD3L4aNnRPyQzfQ7pecq4tPb46PjZVKUnTP9i' as Address
        ), // vrt
      ],
    });

    const jitoSolRewardAmount = 100_000_000_000n; // 100
    await validator.airdropToken(
      vaultAddress,
      'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
      jitoSolRewardAmount
    );

    const fund_1 = await ctx.fund.resolveAccount(true);

    // run operator harvest
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestReward',
      operator: restaking.knownAddresses.fundManager,
    });

    const fund_2 = await ctx.fund.resolveAccount(true);

    const fund_1_jitoSol = fund_1?.data.supportedTokens.filter(
      (supportedToken) =>
        supportedToken.mint == 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn'
    )[0];
    const fund_2_jitoSol = fund_2?.data.supportedTokens.filter(
      (supportedToken) =>
        supportedToken.mint == 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn'
    )[0];

    expect(
      fund_2_jitoSol.token.operationReservedAmount -
        fund_1_jitoSol.token.operationReservedAmount
    ).toEqual(jitoSolRewardAmount);
  });

  test('virtual vault harvest/distribute', async () => {
    const [vaultAddress] = await getProgramDerivedAddress({
      programAddress: restaking.program.address as unknown as Address,
      seeds: [
        getBytesEncoder().encode(Buffer.from('virtual_vault')),
        getAddressEncoder().encode(
          '8vEunBQvD3L4aNnRPyQzfQ7pecq4tPb46PjZVKUnTP9i' as Address
        ), // vrt
      ],
    });

    const zeusRewardAmount = 100_000_000n; // 100
    await validator.airdropToken(
      vaultAddress,
      'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
      zeusRewardAmount
    );

    const globalReward_1 = await ctx.reward.resolve(true);

    // run operator harvest
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestReward',
      operator: restaking.knownAddresses.fundManager,
    });

    const globalReward_2 = await ctx.reward.resolve(true);

    expect(
      globalReward_2?.basePool.settlements[1].blocks[0].amount -
        globalReward_1?.basePool.settlements[1].blocks[0].amount
    ).toEqual(zeusRewardAmount);
  });
});
