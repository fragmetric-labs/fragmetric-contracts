import { afterAll, beforeAll, describe, expect, test } from 'vitest';
import { createTestSuiteContext, expectMasked } from '../../testutil';
import { initializeFrag2 } from './frag2.init';

describe('restaking.frag2 test', async () => {
  const testCtx = await initializeFrag2(await createTestSuiteContext());

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  const { validator, feePayer, restaking, initializationTasks, sdk } = testCtx;
  const ctx = restaking.frag2;

  const [signer1, signer2] = await Promise.all([
    validator
      .newSigner('frag2DepositTestSigner1', 100_000_000_000n)
      .then(async (signer) => {
        await Promise.all([
          validator.airdropToken(
            signer.address,
            'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5',
            100_000_000_000n
          ),
        ]);
        return signer;
      }),
    validator
      .newSigner('frag2DepositTestSigner2', 100_000_000_000n)
      .then(async (signer) => {
        await Promise.all([
          validator.airdropToken(
            signer.address,
            'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5',
            100_000_000_000n
          ),
        ]);
        return signer;
      }),
    validator.airdrop(restaking.knownAddresses.fundManager, 100_000_000_000n),
  ]);
  const user1 = ctx.user(signer1);
  const user2 = ctx.user(signer2);

  /** 1. configuration */
  test('restaking.frag2 initializationTasks snapshot', async () => {
    await expectMasked(initializationTasks).resolves.toMatchSnapshot();
  });

  test('restaking.frag2.resolve', async () => {
    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "MASKED(__lookupTableAddress)",
        "__pricingSources": [
          {
            "address": "4YH3gwg84qRHrPabYj4f7NMVawG5Wd6gM7ZDciuCckTo",
            "role": 0,
          },
          {
            "address": "6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK",
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
            "mint": "VVRTiZKXoPdME1ssmRdzowNG2VFVFG6Rmy9VViXaWa8",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 9,
            "depositable": true,
            "mint": "FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1203870769n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": null,
      }
    `);
  });

  test('restaking.frag2.fund.resolve', async () => {
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
            "tokenMint": "FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5",
            "tokenWithdrawable": true,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
        ],
        "generalStrategy": {
          "depositEnabled": true,
          "donationEnabled": false,
          "operationEnabled": true,
          "transferEnabled": true,
          "withdrawalBatchThresholdSeconds": 1n,
          "withdrawalEnabled": true,
          "withdrawalFeeRateBps": 20,
        },
        "restakingVaultStrategies": [
          {
            "compoundingRewardTokens": [
              {
                "harvestThresholdIntervalSeconds": 0n,
                "harvestThresholdMaxAmount": 18446744073709551615n,
                "harvestThresholdMinAmount": 0n,
                "lastHarvestedAt": 0n,
                "mint": "FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5",
              },
            ],
            "delegations": [],
            "distributingRewardTokens": [
              {
                "harvestThresholdIntervalSeconds": 1n,
                "harvestThresholdMaxAmount": 1000000000000n,
                "harvestThresholdMinAmount": 1000000000n,
                "lastHarvestedAt": 0n,
                "mint": "FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF",
              },
            ],
            "pricingSource": {
              "__kind": "VirtualVault",
              "address": "6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK",
            },
            "rewardCommissionRateBps": 0,
            "solAllocationCapacityAmount": 0n,
            "solAllocationWeight": 0n,
            "vault": "6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK",
            "vaultReceiptTokenDepositable": false,
          },
        ],
        "tokenSwapStrategies": [],
      }
    `);
  });

  test('restaking.frag2.reward.resolve', async () => {
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
                "claimable": true,
                "decimals": 9,
                "description": "Governance vote distribution",
                "id": 0,
                "mint": "FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF",
                "name": "FVT",
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
          "settlements": [],
          "tokenAllocatedAmount": {
            "records": [],
            "totalAmount": 0n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "receiptTokenMint": "DCoj5m7joWjP9T3iPH22q7bDBoGkgUX4ffoL1eQZstwk",
        "rewards": [
          {
            "claimable": true,
            "decimals": 9,
            "description": "Governance vote distribution",
            "id": 0,
            "mint": "FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF",
            "name": "FVT",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          },
        ],
      }
    `);
  });

  /** 2. deposit */
  test('user can deposit frag', async () => {
    await expectMasked(
      user1.deposit.execute(
        {
          assetMint: 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5',
          assetAmount: 5_000_000_000n,
        },
        { signers: [signer1] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": 5000000000n,
          "assetMint": "FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5",
          "metadata": null,
          "skipUserFundAccountCreation": false,
          "skipUserRewardAccountCreation": false,
        },
        "events": {
          "unknown": [],
          "userCreatedOrUpdatedFundAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "DCoj5m7joWjP9T3iPH22q7bDBoGkgUX4ffoL1eQZstwk",
            "user": "FrhgfmDgXvzCmx2zpoYkJN2Xmx3djpQji8eZNzpZEWYY",
            "userFundAccount": "GygWDNHDXdAnbQaDDBTrEK3jfKZ3TzPBZykmvLQkSwkf",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "DCoj5m7joWjP9T3iPH22q7bDBoGkgUX4ffoL1eQZstwk",
            "user": "FrhgfmDgXvzCmx2zpoYkJN2Xmx3djpQji8eZNzpZEWYY",
            "userRewardAccount": "EQn7pu4AaD8anmMHuzMxDnuTxUYGY98cfEBzosGMu9SJ",
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
              "value": "FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5",
            },
            "updatedUserRewardAccounts": [
              "EQn7pu4AaD8anmMHuzMxDnuTxUYGY98cfEBzosGMu9SJ",
            ],
            "user": "FrhgfmDgXvzCmx2zpoYkJN2Xmx3djpQji8eZNzpZEWYY",
            "userFundAccount": "GygWDNHDXdAnbQaDDBTrEK3jfKZ3TzPBZykmvLQkSwkf",
            "userReceiptTokenAccount": "H8zqWqrcdyTaBcpegwQCFPAgKUz2jnJfWn4NH2XW5sJE",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "FKhVzVa7V7LABtY91H9tiYrKgKPZi4NZBh4nXGxj8PoW",
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
            "mint": "FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
          },
        ],
        "user": "FrhgfmDgXvzCmx2zpoYkJN2Xmx3djpQji8eZNzpZEWYY",
        "withdrawalRequests": [],
        "wrappedTokenAmount": 0n,
        "wrappedTokenMint": null,
      }
    `);

    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "MASKED(__lookupTableAddress)",
        "__pricingSources": [
          {
            "address": "4YH3gwg84qRHrPabYj4f7NMVawG5Wd6gM7ZDciuCckTo",
            "role": 0,
          },
          {
            "address": "6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK",
            "role": 0,
          },
        ],
        "depositResidualMicroReceiptTokenAmount": 0n,
        "metadata": null,
        "normalizedToken": null,
        "oneReceiptTokenAsSOL": 1203870769n,
        "receiptTokenDecimals": 9,
        "receiptTokenMint": "DCoj5m7joWjP9T3iPH22q7bDBoGkgUX4ffoL1eQZstwk",
        "receiptTokenSupply": 5000000000n,
        "restakingVaultReceiptTokens": [
          {
            "mint": "VVRTiZKXoPdME1ssmRdzowNG2VFVFG6Rmy9VViXaWa8",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 9,
            "depositable": true,
            "mint": "FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5",
            "oneTokenAsReceiptToken": 1000000000n,
            "oneTokenAsSol": 1203870769n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 5000000000n,
            "operationTotalAmount": 5000000000n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 5000000000n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": null,
      }
    `);
  });

  /** 3. withdraw */
  test('user can withdraw receipt token as frag', async () => {
    await expect(
      user1.requestWithdrawal.execute(
        {
          assetMint: 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5',
          receiptTokenAmount: 1_000_000_000n,
        },
        { signers: [signer1] }
      )
    ).resolves.toMatchObject({
      events: {
        userRequestedWithdrawalFromFund: {
          supportedTokenMint: {
            __option: 'Some',
            value: 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5',
          },
          requestedReceiptTokenAmount: 1_000_000_000n,
        },
      },
    });
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'EnqueueWithdrawalBatch',
    });
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'ProcessWithdrawalBatch',
    });
    await expect(
      ctx.fund
        .resolveAccount(true)
        .then(
          (account) =>
            account?.data.supportedTokens.find(
              (token) =>
                token.mint == 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5'
            )?.token.withdrawalLastProcessedBatchId
        )
    ).resolves.toEqual(1n);

    const res = await user1.withdraw.execute(
      {
        assetMint: 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5',
        requestId: 1n,
      },
      { signers: [signer1] }
    );
    const evt = res.events!.userWithdrewFromFund!;
    expect(
      evt.burntReceiptTokenAmount,
      'burntReceiptTokenAmount = withdrawnAmount + deductedFeeAmount + [optional remainder]'
    ).toBeOneOf([
      evt.withdrawnAmount + evt.deductedFeeAmount,
      evt.withdrawnAmount + evt.deductedFeeAmount + 1n,
    ]);
  });

  /** 4. virtual vault harvest */
  test('virtual vault harvest/compound', async () => {
    const fragRewardAmount = 1_000_000_000n; // 20% of current fund NAV
    await validator.airdropToken(
      '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
      'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5',
      fragRewardAmount
    );

    const fund_1 = await ctx.fund.resolveAccount(true);

    // run operator harvest
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestRestakingYield',
      operator: restaking.knownAddresses.fundManager,
    });

    const fund_2 = await ctx.fund.resolveAccount(true);

    const fund_1_frag = fund_1?.data.supportedTokens.filter(
      (supportedToken) =>
        supportedToken.mint == 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5'
    )[0];
    const fund_2_frag = fund_2?.data.supportedTokens.filter(
      (supportedToken) =>
        supportedToken.mint == 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5'
    )[0];

    expect(
      fund_2_frag!.token.operationReservedAmount -
        fund_1_frag!.token.operationReservedAmount
    ).toEqual(fragRewardAmount);

    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "MASKED(__lookupTableAddress)",
        "__pricingSources": [
          {
            "address": "4YH3gwg84qRHrPabYj4f7NMVawG5Wd6gM7ZDciuCckTo",
            "role": 0,
          },
          {
            "address": "6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK",
            "role": 0,
          },
        ],
        "depositResidualMicroReceiptTokenAmount": 0n,
        "metadata": null,
        "normalizedToken": null,
        "oneReceiptTokenAsSOL": 1504838461n,
        "receiptTokenDecimals": 9,
        "receiptTokenMint": "DCoj5m7joWjP9T3iPH22q7bDBoGkgUX4ffoL1eQZstwk",
        "receiptTokenSupply": 4000000000n,
        "restakingVaultReceiptTokens": [
          {
            "mint": "VVRTiZKXoPdME1ssmRdzowNG2VFVFG6Rmy9VViXaWa8",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 9,
            "depositable": true,
            "mint": "FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5",
            "oneTokenAsReceiptToken": 799999999n,
            "oneTokenAsSol": 1203870769n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 5000000001n,
            "operationTotalAmount": 5000000001n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 4000000000n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 999999n,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": null,
      }
    `);
  });

  test('virtual vault harvest/compound should not occur by compounding threshold', async () => {
    const rewardTokenMint = 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5';

    // 1. reward min amount threshold -> harvest would not occur
    await ctx.fund.updateRestakingVaultRewardHarvestThreshold.execute({
      vault: '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
      rewardTokenMint,
      harvestThresholdMinAmount: 600_000_000n,
      harvestThresholdMaxAmount: 700_000_000n,
      harvestThresholdIntervalSeconds: 1n,
    });

    await validator.airdropToken(
      '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
      'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5',
      500_000_000n
    );

    const fund_2_1 = await ctx.fund.resolveAccount(true);

    const fund_2_1_frag = fund_2_1?.data.supportedTokens.filter(
      (supportedToken) =>
        supportedToken.mint == 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5'
    )[0];

    // run operator harvest
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestRestakingYield',
      operator: restaking.knownAddresses.fundManager,
    });

    const fund_2_2 = await ctx.fund.resolveAccount(true);

    const fund_2_2_frag = fund_2_2?.data.supportedTokens.filter(
      (supportedToken) =>
        supportedToken.mint == 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5'
    )[0];

    expect(fund_2_2_frag!.token.operationReservedAmount).toEqual(
      fund_2_1_frag!.token.operationReservedAmount
    );

    // 2. reward interval second threshold -> harvest would not occur
    await ctx.fund.updateRestakingVaultRewardHarvestThreshold.execute({
      vault: '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
      rewardTokenMint,
      harvestThresholdMinAmount: 200_000_000n,
      harvestThresholdMaxAmount: 600_000_000n,
      harvestThresholdIntervalSeconds: 100n,
    });

    // try to harvest reward
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestRestakingYield',
      operator: restaking.knownAddresses.fundManager,
    });

    const fund_2_3 = await ctx.fund.resolveAccount(true);

    const fund_2_3_frag = fund_2_3?.data.supportedTokens.filter(
      (supportedToken) =>
        supportedToken.mint == 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5'
    )[0];

    expect(fund_2_3_frag!.token.operationReservedAmount).toEqual(
      fund_2_1_frag!.token.operationReservedAmount
    );

    // 3. reward max amount threshold -> only max amount harvested
    await ctx.fund.updateRestakingVaultRewardHarvestThreshold.execute({
      vault: '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
      rewardTokenMint,
      harvestThresholdMinAmount: 200_000_000n,
      harvestThresholdMaxAmount: 400_000_000n,
      harvestThresholdIntervalSeconds: 0n,
    });

    // harvest occurs
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestRestakingYield',
      operator: restaking.knownAddresses.fundManager,
    });

    const fund_3 = await ctx.fund.resolveAccount(true);

    const fund_3_frag = fund_3?.data.supportedTokens.filter(
      (supportedToken) =>
        supportedToken.mint == 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5'
    )[0];

    expect(fund_3_frag!.token.operationReservedAmount).toEqual(
      fund_2_1_frag!.token.operationReservedAmount + 400_000_000n
    );

    // set back to normal state
    await ctx.fund.updateRestakingVaultRewardHarvestThreshold.execute({
      vault: '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
      rewardTokenMint: 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
      harvestThresholdMinAmount: 1_000_000_000n,
      harvestThresholdMaxAmount: 1_000_000_000_000n,
      harvestThresholdIntervalSeconds: 1n,
    });
  });

  test('virtual vault harvest/distribute', async () => {
    const voteRewardAmount = 1_000_000_000n;
    await validator.airdropToken(
      '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
      'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
      voteRewardAmount
    );

    const globalReward_1 = await ctx.reward.resolve(true);

    // run operator harvest
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestRestakingYield',
      operator: restaking.knownAddresses.fundManager,
    });

    const globalReward_2 = await ctx.reward.resolve(true);

    expect(
      globalReward_2!.basePool.settlements[0].blocks[0].amount -
        globalReward_1!.basePool.settlements[0].blocks[0].amount
    ).toEqual(voteRewardAmount);
  });

  /** 5. reward **/
  test('reward settlement clears one block before block addition when block queue is full', async () => {
    // ensure a few blocks filled
    await validator.airdropToken(
      (await ctx.reward.reserve.resolveAddress())!,
      'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
      1000_000_000_000n
    );
    await ctx.reward.settleReward.execute({
      mint: 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
      amount: 100_000_000n,
    });

    const user1ReceiptTokenAmount = await user1.receiptToken
      .resolve(true)
      .then((res) => res!.amount);

    // ensure a new user deposits same amount with old user
    const res = await user2.deposit.execute(
      {
        assetMint: 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5',
        assetAmount: 10_000_000_000n,
      },
      {
        signers: [signer2],
      }
    );
    expect(
      res.events?.userDepositedToFund?.mintedReceiptTokenAmount
    ).toBeGreaterThanOrEqual(user1ReceiptTokenAmount);
    const user2OverMintedAmount =
      res.events!.userDepositedToFund!.mintedReceiptTokenAmount -
      user1ReceiptTokenAmount;
    await user2.requestWithdrawal.execute(
      {
        assetMint: 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5',
        receiptTokenAmount: user2OverMintedAmount,
      },
      {
        signers: [signer2],
      }
    );

    const user2ReceiptTokenAmount = await user2.receiptToken
      .resolve(true)
      .then((res) => res!.amount);
    expect(user2ReceiptTokenAmount).toEqual(user1ReceiptTokenAmount);

    // prepare fresh contribution accounting between two users
    await ctx.reward.settleReward.execute({
      mint: 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
      amount: 0n,
    });

    // settle new fresh 64 blocks to clear stale blocks
    let globalReward = (await ctx.reward.resolve(true))!;
    expect(globalReward.basePool.settlements[0].reward.mint).toEqual(
      'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF'
    );

    const remainingAmount =
      globalReward.basePool.settlements[0].remainingAmount;
    const clearingAmount = globalReward.basePool.settlements[0].blocks.reduce(
      (acc, block) => acc + block.amount,
      0n
    );

    for (let i = 0; i < 64; i++) {
      await validator.skipSlots(10n);
      await ctx.reward.settleReward.execute({
        mint: 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
        amount: 100_000_000n,
      });
    }

    globalReward = (await ctx.reward.resolve(true))!;
    expect(globalReward.basePool.settlements[0].remainingAmount).toEqual(
      remainingAmount + clearingAmount
    );
    expect(globalReward.basePool.settlements[0].blocks.length).toEqual(64);

    // only the old user lost the reward for the stale blocks portion.
    await user1.reward.claim.execute(
      {
        mint: 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
        amount: null,
        recipient: null,
      },
      {
        signers: [signer1],
      }
    );
    await expectMasked(user1.reward.resolve(true)).resolves
      .toMatchInlineSnapshot(`
      {
        "basePool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "settlements": [
            {
              "claimedAmount": 3200000000n,
              "reward": {
                "claimable": true,
                "decimals": 9,
                "description": "Governance vote distribution",
                "id": 0,
                "mint": "FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF",
                "name": "FVT",
                "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
              },
              "settledAmount": 3200000000n,
              "settledContribution": "MASKED(/[.*C|c]ontribution?$/)",
              "settledSlot": "MASKED(/[.*S|s]lots?$/)",
            },
          ],
          "tokenAllocatedAmount": {
            "records": [
              {
                "amount": 4000000000n,
                "contributionAccrualRate": 1,
              },
            ],
            "totalAmount": 4000000000n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "bonusPool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "settlements": [],
          "tokenAllocatedAmount": {
            "records": [
              {
                "amount": 4000000000n,
                "contributionAccrualRate": 1,
              },
            ],
            "totalAmount": 4000000000n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "delegate": null,
        "receiptTokenMint": "DCoj5m7joWjP9T3iPH22q7bDBoGkgUX4ffoL1eQZstwk",
        "user": "FrhgfmDgXvzCmx2zpoYkJN2Xmx3djpQji8eZNzpZEWYY",
      }
    `);

    await user2.reward.claim.execute(
      {
        mint: 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
        amount: null,
        recipient: null,
      },
      {
        signers: [signer2],
      }
    );
    await expectMasked(user2.reward.resolve(true)).resolves
      .toMatchInlineSnapshot(`
      {
        "basePool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "settlements": [
            {
              "claimedAmount": 3200000000n,
              "reward": {
                "claimable": true,
                "decimals": 9,
                "description": "Governance vote distribution",
                "id": 0,
                "mint": "FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF",
                "name": "FVT",
                "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
              },
              "settledAmount": 3200000000n,
              "settledContribution": "MASKED(/[.*C|c]ontribution?$/)",
              "settledSlot": "MASKED(/[.*S|s]lots?$/)",
            },
          ],
          "tokenAllocatedAmount": {
            "records": [
              {
                "amount": 4000000000n,
                "contributionAccrualRate": 1,
              },
            ],
            "totalAmount": 4000000000n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "bonusPool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "settlements": [],
          "tokenAllocatedAmount": {
            "records": [
              {
                "amount": 4000000000n,
                "contributionAccrualRate": 1,
              },
            ],
            "totalAmount": 4000000000n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "delegate": null,
        "receiptTokenMint": "DCoj5m7joWjP9T3iPH22q7bDBoGkgUX4ffoL1eQZstwk",
        "user": "8Ghhx1z1VCgLZAeFRjJhLy4kq15GrQzrZk71qKSok3vk",
      }
    `);

    expect(
      globalReward.basePool.settlements[0]
        .settlementBlocksLastRewardPoolContribution
    ).toEqual(
      user1.reward.account!.data.baseUserRewardPool.rewardSettlements1[0]
        .totalSettledContribution +
        user2.reward.account!.data.baseUserRewardPool.rewardSettlements1[0]
          .totalSettledContribution
    );
  });

  test('leftovers from cleared blocks are claimed to program revenue account', async () => {
    let globalReward = (await ctx.reward.resolve(true))!;
    const remainingAmount =
      globalReward.basePool.settlements[0].remainingAmount;
    const claimedAmount = globalReward.basePool.settlements[0].claimedAmount;
    expect(remainingAmount).toBeGreaterThan(0n);

    const rewardIndex = globalReward.rewards.findIndex(
      (reward) => reward.mint == 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF'
    );

    const rewardReserveAccount = ctx.user(ctx.reward.reserve.address!);
    await rewardReserveAccount.rewardTokens.resolve(true);
    const rewardTokenReserveAccount =
      rewardReserveAccount.rewardTokens.children[rewardIndex];

    let rewardTokenReserveAccountBalance = await rewardTokenReserveAccount
      .resolve(true)
      .then((res) => res!.amount);

    const programRevenueAccount = ctx.user(
      'GuSruSKKCmAGuWMeMsiw3mbNhjeiRtNhnh9Eatgz33NA'
    );
    await programRevenueAccount.rewardTokens.resolve(true);
    const programRewardTokenRevenueAccount =
      programRevenueAccount.rewardTokens.children[rewardIndex];

    let programRewardTokenRevenueAccountBalance =
      await programRewardTokenRevenueAccount
        .resolve(true)
        .then((res) => res?.amount ?? 0n);

    await ctx.reward.claimRemainingReward.execute({
      mint: 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
    });

    globalReward = (await ctx.reward.resolve(true))!;
    expect(globalReward.basePool.settlements[0].remainingAmount).toEqual(0n);
    expect(globalReward.basePool.settlements[0].claimedAmount).toEqual(
      remainingAmount + claimedAmount
    );

    await expect(
      rewardTokenReserveAccount.resolve(true).then((res) => res!.amount)
    ).resolves.toEqual(rewardTokenReserveAccountBalance - remainingAmount);
    await expect(
      programRewardTokenRevenueAccount.resolve(true).then((res) => res!.amount)
    ).resolves.toEqual(
      programRewardTokenRevenueAccountBalance + remainingAmount
    );
  });

  test('operator also claims leftovers from cleared block after settle', async () => {
    // ensure remaining amount > 0
    await ctx.reward.settleReward.execute({
      mint: 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
      amount: 1n,
    });
    await user1.reward.updatePools.execute({});
    await user2.reward.updatePools.execute({});
    await ctx.reward.updatePools.execute({});

    let globalReward = (await ctx.reward.resolve(true))!;
    const remainingAmount =
      globalReward.basePool.settlements[0].remainingAmount;
    const claimedAmount = globalReward.basePool.settlements[0].claimedAmount;
    const numBlocks = globalReward.basePool.settlements[0].blocks.length;
    const clearingBlock =
      numBlocks == 64 ? globalReward.basePool.settlements[0].blocks[0] : null;
    const clearingBlockRemainingAmount = clearingBlock
      ? clearingBlock.amount - clearingBlock.userSettledAmount
      : 0n;
    expect(remainingAmount + clearingBlockRemainingAmount).toBeGreaterThan(0n);

    const voteRewardAmount = 1_000_000_000n;
    await validator.airdropToken(
      '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
      'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
      voteRewardAmount
    );

    const rewardIndex = globalReward.rewards.findIndex(
      (reward) => reward.mint == 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF'
    );

    const rewardReserveAccount = ctx.user(ctx.reward.reserve.address!);
    await rewardReserveAccount.rewardTokens.resolve(true);
    const rewardTokenReserveAccount =
      rewardReserveAccount.rewardTokens.children[rewardIndex];

    let rewardTokenReserveAccountBalance = await rewardTokenReserveAccount
      .resolve(true)
      .then((res) => res!.amount);

    const programRevenueAccount = ctx.user(
      'GuSruSKKCmAGuWMeMsiw3mbNhjeiRtNhnh9Eatgz33NA'
    );
    await programRevenueAccount.rewardTokens.resolve(true);
    const programRewardTokenRevenueAccount =
      programRevenueAccount.rewardTokens.children[rewardIndex];

    let programRewardTokenRevenueAccountBalance =
      await programRewardTokenRevenueAccount
        .resolve(true)
        .then((res) => res?.amount ?? 0n);

    // run operator harvest
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestRestakingYield',
      operator: restaking.knownAddresses.fundManager,
    });

    globalReward = (await ctx.reward.resolve(true))!;
    expect(globalReward.basePool.settlements[0].remainingAmount).toEqual(0n);
    expect(globalReward.basePool.settlements[0].claimedAmount).toEqual(
      remainingAmount + claimedAmount + clearingBlockRemainingAmount
    );

    await expect(
      rewardTokenReserveAccount.resolve(true).then((res) => res!.amount)
    ).resolves.toEqual(
      rewardTokenReserveAccountBalance +
        voteRewardAmount -
        remainingAmount -
        clearingBlockRemainingAmount
    );
    await expect(
      programRewardTokenRevenueAccount.resolve(true).then((res) => res!.amount)
    ).resolves.toEqual(
      programRewardTokenRevenueAccountBalance +
        remainingAmount +
        clearingBlockRemainingAmount
    );
  });

  test('reward is transferred to revenue account based on commission rate during harvest command execution (compound reward, distribute reward)', async () => {
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestRestakingYield',
      operator: restaking.knownAddresses.fundManager,
    });

    // loosen compound reward harvest threshold
    await ctx.fund.updateRestakingVaultRewardHarvestThreshold.execute({
      vault: '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
      rewardTokenMint: 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5',
      harvestThresholdMinAmount: 0n,
      harvestThresholdMaxAmount: 18_446_744_073_709_551_615n,
      harvestThresholdIntervalSeconds: 0n,
    });

    // loosen distribute reward harvest threshold
    await ctx.fund.updateRestakingVaultRewardHarvestThreshold.execute({
      vault: '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
      rewardTokenMint: 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
      harvestThresholdMinAmount: 0n,
      harvestThresholdMaxAmount: 18_446_744_073_709_551_615n,
      harvestThresholdIntervalSeconds: 0n,
    });

    await expectMasked(ctx.fund.resolve(true)).resolves.toMatchInlineSnapshot(`
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
            "tokenAccumulatedDepositAmount": 15000000000n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5",
            "tokenWithdrawable": true,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
        ],
        "generalStrategy": {
          "depositEnabled": true,
          "donationEnabled": false,
          "operationEnabled": true,
          "transferEnabled": true,
          "withdrawalBatchThresholdSeconds": 1n,
          "withdrawalEnabled": true,
          "withdrawalFeeRateBps": 20,
        },
        "restakingVaultStrategies": [
          {
            "compoundingRewardTokens": [
              {
                "harvestThresholdIntervalSeconds": 0n,
                "harvestThresholdMaxAmount": 18446744073709551615n,
                "harvestThresholdMinAmount": 0n,
                "lastHarvestedAt": "MASKED(/.*At?$/)",
                "mint": "FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5",
              },
            ],
            "delegations": [],
            "distributingRewardTokens": [
              {
                "harvestThresholdIntervalSeconds": 0n,
                "harvestThresholdMaxAmount": 18446744073709551615n,
                "harvestThresholdMinAmount": 0n,
                "lastHarvestedAt": "MASKED(/.*At?$/)",
                "mint": "FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF",
              },
            ],
            "pricingSource": {
              "__kind": "VirtualVault",
              "address": "6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK",
            },
            "rewardCommissionRateBps": 0,
            "solAllocationCapacityAmount": 0n,
            "solAllocationWeight": 0n,
            "vault": "6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK",
            "vaultReceiptTokenDepositable": false,
          },
        ],
        "tokenSwapStrategies": [],
      }
    `);

    const programRevenueFragTokenAccount =
      sdk.TokenAccountContext.fromAssociatedTokenSeeds(restaking, () =>
        Promise.resolve({
          owner: 'GuSruSKKCmAGuWMeMsiw3mbNhjeiRtNhnh9Eatgz33NA',
          mint: 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5',
        })
      );

    const programRevenueFragVoteTokenAccount =
      sdk.TokenAccountContext.fromAssociatedTokenSeeds(restaking, () =>
        Promise.resolve({
          owner: 'GuSruSKKCmAGuWMeMsiw3mbNhjeiRtNhnh9Eatgz33NA',
          mint: 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
        })
      );

    const MAX_REWARD_COMMISSION_RATE_BPS = 10000;

    for (
      let rewardCommissionRateBps = 0;
      rewardCommissionRateBps <= MAX_REWARD_COMMISSION_RATE_BPS;
      rewardCommissionRateBps += 520
    ) {
      await ctx.fund.runCommand.executeChained({
        forceResetCommand: 'HarvestRestakingYield',
        operator: restaking.knownAddresses.fundManager,
      });

      await ctx.fund.updateRestakingVaultStrategy.execute({
        vault: '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
        rewardCommissionRateBps: rewardCommissionRateBps as unknown as number,
      });

      // 1) compound reward (frag Token)
      const fragTokenStatusBefore = await ctx.fund
        .resolveAccount(true)
        .then(
          (fundAccount) =>
            fundAccount!.data.supportedTokens.filter(
              (supportedToken) =>
                supportedToken.mint ==
                'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5'
            )[0]
        );
      const fragTokenAmountBefore =
        fragTokenStatusBefore!.token.operationReservedAmount;

      const programRevenueCompoundRewardTokenAmountBefore =
        await programRevenueFragTokenAccount
          .resolveAccount(true)
          .then((account) => (account ? account.data.amount : 0n));

      // airdrop compound reward (frag token)
      await validator.airdropToken(
        '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
        'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5',
        5_000_000_000_000n
      );

      // harvest compounding reward
      await ctx.fund.runCommand.executeChained({
        forceResetCommand: 'HarvestRestakingYield',
        operator: restaking.knownAddresses.fundManager,
      });

      const fragTokenStatusAfter = await ctx.fund
        .resolveAccount(true)
        .then(
          (fundAccount) =>
            fundAccount!.data.supportedTokens.filter(
              (supportedToken) =>
                supportedToken.mint ==
                'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5'
            )[0]
        );
      const fragTokenAmountAfter =
        fragTokenStatusAfter!.token.operationReservedAmount;

      const programRevenueCompoundRewardTokenAmountAfter =
        await programRevenueFragTokenAccount
          .resolveAccount(true)
          .then((account) => (account ? account.data.amount : 0n));

      const programRevenueCompoundRewardTokenAmountDelta =
        programRevenueCompoundRewardTokenAmountAfter -
        programRevenueCompoundRewardTokenAmountBefore;
      const supportedTokenAccountBalanceDelta =
        fragTokenAmountAfter - fragTokenAmountBefore;

      expect(programRevenueCompoundRewardTokenAmountDelta).toEqual(
        (5_000_000_000_000n * BigInt(rewardCommissionRateBps)) / 10000n
      );
      expect(
        programRevenueCompoundRewardTokenAmountDelta +
          supportedTokenAccountBalanceDelta
      ).toEqual(5_000_000_000_000n);

      // 2) distribute reward (frag vote Token)
      const fragVoteTokenAmountBefore = (
        await ctx.reward.reserve.rewardTokens.resolve(true)
      ).filter(
        (rewardToken) =>
          rewardToken.mint == 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF'
      )[0].amount;

      const programRevenueDistributeRewardTokenAmountBefore =
        await programRevenueFragVoteTokenAccount
          .resolveAccount(true)
          .then((account) => (account ? account.data.amount : 0n));

      // airdrop distribute reward (frag vote token)
      await validator.airdropToken(
        '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
        'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF',
        123_456_789_987_654_321n
      );

      // harvest distributing reward
      await ctx.fund.runCommand.executeChained({
        forceResetCommand: 'HarvestRestakingYield',
        operator: restaking.knownAddresses.fundManager,
      });

      const fragVoteTokenAmountAfter = (
        await ctx.reward.reserve.rewardTokens.resolve(true)
      ).filter(
        (rewardToken) =>
          rewardToken.mint == 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF'
      )[0].amount;

      const programRevenueDistributeRewardTokenAmountAfter =
        await programRevenueFragVoteTokenAccount
          .resolveAccount(true)
          .then((account) => (account ? account.data.amount : 0n));

      const programRevenueDistributeRewardTokenAmountDelta =
        programRevenueDistributeRewardTokenAmountAfter -
        programRevenueDistributeRewardTokenAmountBefore;
      const rewardTokenAccountBalanceDelta = BigInt(
        fragVoteTokenAmountAfter - fragVoteTokenAmountBefore
      );

      expect(programRevenueDistributeRewardTokenAmountDelta).toEqual(
        (123_456_789_987_654_321n * BigInt(rewardCommissionRateBps)) / 10000n
      );
      expect(
        programRevenueDistributeRewardTokenAmountDelta +
          rewardTokenAccountBalanceDelta
      ).toEqual(123_456_789_987_654_321n);
    }

    // hard limit test (reward commission bps <= 100%)
    await expect(
      ctx.fund.updateRestakingVaultStrategy.execute({
        vault: '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
        rewardCommissionRateBps: MAX_REWARD_COMMISSION_RATE_BPS,
      })
    ).resolves.not.toThrow();

    await expect(
      ctx.fund.updateRestakingVaultStrategy.execute({
        vault: '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
        rewardCommissionRateBps: MAX_REWARD_COMMISSION_RATE_BPS + 1,
      })
    ).rejects.toThrow();

    // reset to 0
    await expect(
      ctx.fund.updateRestakingVaultStrategy.execute({
        vault: '6f4bndUq1ct6s7QxiHFk98b1Q7JdJw3zTTZBGbSPP6gK',
        rewardCommissionRateBps: 0,
      })
    ).resolves.not.toThrow();
  });

  /** 6. Operation */
  test('run full operation cycle for regression', async () => {
    // frag2 operation will only initialize -> enqueue withdrawal -> process withdrawal.
    // however this test case is to prevent breaking changes in other commands that affects the full cycle.
    // For example, due to virtual vault's edge case, restaking-related command might be broken unless properly handled
    await ctx.fund.runCommand.executeChained(null);
  });
});
