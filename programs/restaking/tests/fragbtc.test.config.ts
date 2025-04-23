import { expect, test } from 'vitest';
import { initializeFragBTC } from './fragbtc';
import { expectMasked } from './utils';

export const fragBTCConfigurationTest = async (
  testCtx: ReturnType<typeof initializeFragBTC>
) => {
  const { validator, feePayer, restaking, initializationTasks } = testCtx;
  const ctx = restaking.fragBTC;

  test(`restaking.fragBTC initializationTasks snapshot`, async () => {
    await expectMasked(initializationTasks).resolves.toMatchSnapshot();
  });

  test(`restaking.fragBTC.resolve`, async () => {
    await expect(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "G45gQa12Uwvnrp2Yb9oWTSwZSEHZWL71QDWvyLz23bNc",
        "__pricingSources": [
          {
            "address": "4yp9YAXCJsKWMDZq2Q4j4amktvJGXBCpr3Lmv2cYBrb8",
            "role": 0,
          },
        ],
        "metadata": null,
        "oneReceiptTokenAsSOL": 0n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "receiptTokenSupply": 0n,
        "supportedAssets": [
          {
            "decimals": 8,
            "depositable": true,
            "mint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": 1970-01-01T00:00:00.000Z,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": 1970-01-01T00:00:00.000Z,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);
  });

  test('restaking.fragBTC.fund.resolve', async () => {
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
            "tokenMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": true,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 0n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": true,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
        ],
        "generalStrategy": {
          "depositEnabled": true,
          "donationEnabled": true,
          "transferEnabled": true,
          "withdrawalBatchThresholdSeconds": 1n,
          "withdrawalEnabled": true,
          "withdrawalFeeRateBps": 20,
        },
        "restakingVaultStrategies": [],
        "tokenSwapStrategies": [],
      }
    `);
  });

  test(`restaking.fragBTC.reward.resolve`, async () => {
    await expectMasked(ctx.reward.resolve(true)).resolves
      .toMatchInlineSnapshot(`
      {
        "basePool": {
          "contribution": 0n,
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
          "contribution": 0n,
          "customContributionAccrualRateEnabled": true,
          "initialSlot": "MASKED(/[.*S|s]lots?$/)",
          "settlements": [
            {
              "blocks": [
                {
                  "amount": 0n,
                  "endingContribution": 0n,
                  "endingSlot": "MASKED(/[.*S|s]lots?$/)",
                  "settledContribution": 0n,
                  "settledSlots": "MASKED(/[.*S|s]lots?$/)",
                  "startingContribution": 0n,
                  "startingSlot": "MASKED(/[.*S|s]lots?$/)",
                  "userSettledAmount": 0n,
                  "userSettledContribution": 0n,
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
              "settlementBlocksLastRewardPoolContribution": 0n,
              "settlementBlocksLastSlot": "MASKED(/[.*S|s]lots?$/)",
            },
          ],
          "tokenAllocatedAmount": {
            "records": [],
            "totalAmount": 0n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
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
        ],
      }
    `);
  });
};
