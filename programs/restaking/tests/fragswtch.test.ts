import { afterAll, beforeAll, describe, expect, test } from 'vitest';
import { createTestSuiteContext, expectMasked } from '../../testutil';
import { initializeFragSwitch } from './fragswtch.init';

describe('restaking.fragSWTCH test', async () => {
  const testCtx = await initializeFragSwitch(await createTestSuiteContext());

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  const { validator, feePayer, restaking, initializationTasks, sdk } = testCtx;
  const ctx = restaking.fragSWTCH;

  const [signer1, signer2] = await Promise.all([
    validator
      .newSigner('fragSwitchDepositTestSigner1', 100_000_000_000n)
      .then(async (signer) => {
        await Promise.all([
          validator.airdropToken(
            signer.address,
            'SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx',
            100_000_000_000n
          ),
        ]);
        return signer;
      }),
    validator
      .newSigner('fragSwitchDepositTestSigner2', 100_000_000_000n)
      .then(async (signer) => {
        await Promise.all([
          validator.airdropToken(
            signer.address,
            'SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx',
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
  test('restaking.fragSWTCH initializationTasks snapshot', async () => {
    await expectMasked(initializationTasks).resolves.toMatchSnapshot();
  });

  test('restaking.fragSWTCH.resolve', async () => {
    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "MASKED(__lookupTableAddress)",
        "__pricingSources": [
          {
            "address": "6urvuvb6XxjqVgCzEaHMTVJ8PkS2PQ91at455Vgt89ME",
            "role": 0,
          },
          {
            "address": "9oDtsX1hoKCG31VAVKCEmTEEbjgDa6fq6vVHyWAVzWV4",
            "role": 0,
          },
        ],
        "depositResidualMicroReceiptTokenAmount": 0n,
        "metadata": null,
        "normalizedToken": null,
        "oneReceiptTokenAsSOL": 0n,
        "receiptTokenDecimals": 9,
        "receiptTokenMint": "DB1mkPYhGBfBKpYMJ3Z4ST4d7oDfQqg3WfnMugCC3fmm",
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
            "vault": "9oDtsX1hoKCG31VAVKCEmTEEbjgDa6fq6vVHyWAVzWV4",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 9,
            "depositable": true,
            "mint": "SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 2777777n,
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

  test('restaking.fragSWTCH.fund.resolve', async () => {
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
            "tokenMint": "SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx",
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
                "harvestThresholdIntervalSeconds": 1n,
                "harvestThresholdMaxAmount": 1000000000000n,
                "harvestThresholdMinAmount": 1000000000n,
                "lastHarvestedAt": 0n,
                "mint": "SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx",
              },
            ],
            "delegations": [],
            "distributingRewardTokens": [],
            "pricingSource": {
              "__kind": "VirtualVault",
              "address": "9oDtsX1hoKCG31VAVKCEmTEEbjgDa6fq6vVHyWAVzWV4",
            },
            "rewardCommissionRateBps": 0,
            "solAllocationCapacityAmount": 0n,
            "solAllocationWeight": 0n,
            "vault": "9oDtsX1hoKCG31VAVKCEmTEEbjgDa6fq6vVHyWAVzWV4",
            "vaultReceiptTokenDepositable": false,
          },
        ],
        "tokenSwapStrategies": [],
      }
    `);
  });

  test('restaking.fragSWTCH.reward.resolve', async () => {
    await expectMasked(ctx.reward.resolve(true)).resolves
      .toMatchInlineSnapshot(`
      {
        "basePool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "customContributionAccrualRateEnabled": false,
          "initialSlot": "MASKED(/[.*S|s]lots?$/)",
          "settlements": [],
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
        "receiptTokenMint": "DB1mkPYhGBfBKpYMJ3Z4ST4d7oDfQqg3WfnMugCC3fmm",
        "rewards": [],
      }
    `);
  });

  /** 2. deposit */
  test('user can deposit swtch', async () => {
    await expectMasked(
      user1.deposit.execute(
        {
          assetMint: 'SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx',
          assetAmount: 5_000_000_000n,
        },
        { signers: [signer1] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": 5000000000n,
          "assetMint": "SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx",
          "metadata": null,
          "skipUserFundAccountCreation": false,
          "skipUserRewardAccountCreation": false,
        },
        "events": {
          "unknown": [],
          "userCreatedOrUpdatedFundAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "DB1mkPYhGBfBKpYMJ3Z4ST4d7oDfQqg3WfnMugCC3fmm",
            "user": "Gy54dp65Lk6yhb5njt89kE2LxBFUARSB6M6DR3fxpSL6",
            "userFundAccount": "A9H93BGNpLMMEUswpvuJkuZtLsSvq4FH8YCgosYZw7T4",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "DB1mkPYhGBfBKpYMJ3Z4ST4d7oDfQqg3WfnMugCC3fmm",
            "user": "Gy54dp65Lk6yhb5njt89kE2LxBFUARSB6M6DR3fxpSL6",
            "userRewardAccount": "46kZ3aJPPzM1BGCt4xUYXoVu85gT58Cy9K4HeRNeNqeV",
          },
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 5000000000n,
            "fundAccount": "2KmQDnhrWiNovV2qqv6qxx7tAPA9uT5k6qSw5nq8HXPW",
            "mintedReceiptTokenAmount": 5000000000n,
            "receiptTokenMint": "DB1mkPYhGBfBKpYMJ3Z4ST4d7oDfQqg3WfnMugCC3fmm",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx",
            },
            "updatedUserRewardAccounts": [
              "46kZ3aJPPzM1BGCt4xUYXoVu85gT58Cy9K4HeRNeNqeV",
            ],
            "user": "Gy54dp65Lk6yhb5njt89kE2LxBFUARSB6M6DR3fxpSL6",
            "userFundAccount": "A9H93BGNpLMMEUswpvuJkuZtLsSvq4FH8YCgosYZw7T4",
            "userReceiptTokenAccount": "3AF5ExYAK89diHwLBfQFu81Qfo58mpvYiLuf7a8sTmdt",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "85ZTYhBaamjq96kayQg5GorLhWPPTe6uDrrq8Hsqfvgh",
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
        "receiptTokenMint": "DB1mkPYhGBfBKpYMJ3Z4ST4d7oDfQqg3WfnMugCC3fmm",
        "supportedAssets": [
          {
            "amount": 95000000000n,
            "decimals": 9,
            "depositable": true,
            "mint": "SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
          },
        ],
        "user": "Gy54dp65Lk6yhb5njt89kE2LxBFUARSB6M6DR3fxpSL6",
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
            "address": "6urvuvb6XxjqVgCzEaHMTVJ8PkS2PQ91at455Vgt89ME",
            "role": 0,
          },
          {
            "address": "9oDtsX1hoKCG31VAVKCEmTEEbjgDa6fq6vVHyWAVzWV4",
            "role": 0,
          },
        ],
        "depositResidualMicroReceiptTokenAmount": 0n,
        "metadata": null,
        "normalizedToken": null,
        "oneReceiptTokenAsSOL": 2777777n,
        "receiptTokenDecimals": 9,
        "receiptTokenMint": "DB1mkPYhGBfBKpYMJ3Z4ST4d7oDfQqg3WfnMugCC3fmm",
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
            "vault": "9oDtsX1hoKCG31VAVKCEmTEEbjgDa6fq6vVHyWAVzWV4",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 9,
            "depositable": true,
            "mint": "SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx",
            "oneTokenAsReceiptToken": 1000000000n,
            "oneTokenAsSol": 2777777n,
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
  test('user can withdraw receipt token as swtch', async () => {
    await expect(
      user1.requestWithdrawal.execute(
        {
          assetMint: 'SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx',
          receiptTokenAmount: 1_000_000_000n,
        },
        { signers: [signer1] }
      )
    ).resolves.toMatchObject({
      events: {
        userRequestedWithdrawalFromFund: {
          supportedTokenMint: {
            __option: 'Some',
            value: 'SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx',
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
                token.mint == 'SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx'
            )?.token.withdrawalLastProcessedBatchId
        )
    ).resolves.toEqual(1n);

    const res = await user1.withdraw.execute(
      {
        assetMint: 'SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx',
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
      '9oDtsX1hoKCG31VAVKCEmTEEbjgDa6fq6vVHyWAVzWV4',
      'SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx',
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
        supportedToken.mint == 'SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx'
    )[0];
    const fund_2_frag = fund_2?.data.supportedTokens.filter(
      (supportedToken) =>
        supportedToken.mint == 'SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx'
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
            "address": "6urvuvb6XxjqVgCzEaHMTVJ8PkS2PQ91at455Vgt89ME",
            "role": 0,
          },
          {
            "address": "9oDtsX1hoKCG31VAVKCEmTEEbjgDa6fq6vVHyWAVzWV4",
            "role": 0,
          },
        ],
        "depositResidualMicroReceiptTokenAmount": 0n,
        "metadata": null,
        "normalizedToken": null,
        "oneReceiptTokenAsSOL": 3472222n,
        "receiptTokenDecimals": 9,
        "receiptTokenMint": "DB1mkPYhGBfBKpYMJ3Z4ST4d7oDfQqg3WfnMugCC3fmm",
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
            "vault": "9oDtsX1hoKCG31VAVKCEmTEEbjgDa6fq6vVHyWAVzWV4",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 9,
            "depositable": true,
            "mint": "SW1TCHLmRGTfW5xZknqQdpdarB8PD95sJYWpNp9TbFx",
            "oneTokenAsReceiptToken": 799999999n,
            "oneTokenAsSol": 2777777n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 5000000001n,
            "operationTotalAmount": 5000000001n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 4000000000n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 999936n,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": null,
      }
    `);
  });

  /** 5. Operation */
  test('run full operation cycle for regression', async () => {
    // fragSWTCH operation will only initialize -> enqueue withdrawal -> process withdrawal.
    // however this test case is to prevent breaking changes in other commands that affects the full cycle.
    // For example, due to virtual vault's edge case, restaking-related command might be broken unless properly handled
    await ctx.fund.runCommand.executeChained(null);
  });
});
