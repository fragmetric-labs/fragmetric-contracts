import { expect, test } from 'vitest';
import { expectMasked } from '../../testutil';
import { initializeFragBTC } from './fragbtc';

export const fragBTCOperationTest = async (
  testCtx: ReturnType<typeof initializeFragBTC>
) => {
  const { validator, feePayer, restaking, initializationTasks } = testCtx;
  const ctx = restaking.fragBTC;

  test('fund with SolvBTCVaults should fully operate', async () => {
    await expectMasked(ctx.fund.runCommand.executeChained(null)).resolves
      .toMatchInlineSnapshot(`
      {
        "args": {
          "forceResetCommand": null,
          "operator": null,
        },
        "events": {
          "operatorRanFundCommand": {
            "command": {
              "__kind": "DelegateVST",
              "fields": [
                {
                  "state": {
                    "__kind": "Prepare",
                    "vaults": [
                      "E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B",
                    ],
                  },
                },
              ],
            },
            "fundAccount": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
            "nextSequence": 0,
            "numOperated": 28n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "result": {
              "__option": "None",
            },
          },
          "unknown": [],
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      __pricingSources: [
        {
          address: '4yp9YAXCJsKWMDZq2Q4j4amktvJGXBCpr3Lmv2cYBrb8',
        },
        {
          address: 'H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6',
        },
        {
          address: '5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ',
        },
        {
          address: 'E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B',
        },
      ],
    });

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
            "tokenAccumulatedDepositAmount": 2322222208n,
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
            "tokenAccumulatedDepositAmount": 1581481477n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": true,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 0n,
            "tokenAccumulatedDepositAmount": 1851851835n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
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
            "compoundingRewardTokenMints": [],
            "delegations": [],
            "distributingRewardTokenMints": [
              "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
            ],
            "pricingSource": {
              "__kind": "SolvBTCVault",
              "address": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
            },
            "solAllocationCapacityAmount": 0n,
            "solAllocationWeight": 0n,
            "vault": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
          },
          {
            "compoundingRewardTokenMints": [],
            "delegations": [],
            "distributingRewardTokenMints": [],
            "pricingSource": {
              "__kind": "SolvBTCVault",
              "address": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
            },
            "solAllocationCapacityAmount": 0n,
            "solAllocationWeight": 0n,
            "vault": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
          },
          {
            "compoundingRewardTokenMints": [],
            "delegations": [],
            "distributingRewardTokenMints": [],
            "pricingSource": {
              "__kind": "SolvBTCVault",
              "address": "E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B",
            },
            "solAllocationCapacityAmount": 0n,
            "solAllocationWeight": 0n,
            "vault": "E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B",
          },
        ],
        "tokenSwapStrategies": [],
      }
    `);
  });
};
