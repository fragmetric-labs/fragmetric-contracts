import { expect, test } from 'vitest';
import { initializeFragJTO } from './fragjto';
import { expectMasked } from './utils';

export const fragJTOConfigurationTest = async (
  testCtx: ReturnType<typeof initializeFragJTO>
) => {
  const { validator, feePayer, restaking, initializationTasks } = testCtx;
  const ctx = restaking.fragJTO;

  test(`restaking.fragJTO initializationTasks snapshot`, async () => {
    await expectMasked(initializationTasks).resolves.toMatchSnapshot();
  });

  test(`restaking.fragJTO.resolve`, async () => {
    await expect(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "G45gQa12Uwvnrp2Yb9oWTSwZSEHZWL71QDWvyLz23bNc",
        "__pricingSources": [
          {
            "address": "2UhFnySoJi6c89aydGAGS7ZRemo2dbkFRhvSJqDX4gHJ",
            "role": 0,
          },
          {
            "address": "BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd",
            "role": 0,
          },
        ],
        "metadata": null,
        "receiptTokenDecimals": 9,
        "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
        "receiptTokenSupply": 0n,
        "supportedAssets": [
          {
            "decimals": 9,
            "depositable": true,
            "mint": "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 14476690n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
            "withdrawalLastBatchProcessedAt": 1970-01-01T00:00:00.000Z,
          },
        ],
        "wrappedTokenMint": "EAvS1wFjAccNpDYbAkW2dwUDEiC7BMvWzwUj2tjRUkHA",
      }
    `);
  });

  test('restaking.fragJTO.fund.resolve', async () => {
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
            "tokenMint": "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
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
          "withdrawalFeeRateBps": 10,
        },
        "restakingVaultStrategies": [
          {
            "compoundingRewardTokenMints": [
              "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            ],
            "delegations": [
              {
                "operator": "FzZ9EXmHv7ANCXijpALUBzCza6wYNprnsfaEHuoNx9sE",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 90n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "LKFpfXtBkH5b7D9mo8dPcjCLZCZpmLQC9ELkbkyVdah",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 90n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "GZxp4e2Tm3Pw9GyAaxuF6odT3XkRM96jpZkp3nxhoK4Y",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 90n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "CA8PaNSoFWzvbCJ2oK3QxBEutgyHSTT5omEptpj8YHPY",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 92n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "7yofWXChEHkPTSnyFdKx2Smq5iWVbGB4P1dkdC6zHWYR",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 90n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "29rxXT5zbTR1ctiooHtb1Sa1TD4odzhQHsrLz3D78G5w",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 90n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "BFEsrxFPsBcY2hR5kgyfKnpwgEc8wYQdngvRukLQXwG2",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 90n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "2sHNuid4rus4sK2EmndLeZcPNKkgzuEoc8Vro3PH2qop",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 0n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "5TGRFaLy3eF93pSNiPamCgvZUN3gzdYcs7jA3iCAsd1L",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 90n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "6AxtdRGAaiAyqcwxVBHsH3xtqCbQuffaiE4epT4koTxk",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 80n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "C6AF8qGCo2dL815ziRCmfdbFeL5xbRLuSTSZzTGBH68y",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 100n,
                "tokenRedelegatingAmount": 0n,
              },
            ],
            "distributingRewardTokenMints": [
              "REALSWTCH7J8JdafNBLZpfSCLiFwpMCqod2RpkU4RNn",
            ],
            "pricingSource": {
              "__kind": "JitoRestakingVault",
              "address": "BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd",
            },
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "vault": "BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd",
          },
        ],
        "tokenSwapStrategies": [
          {
            "fromTokenMint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "swapSource": {
              "__kind": "OrcaDEXLiquidityPool",
              "address": "G2FiE1yn9N9ZJx5e1E2LxxMnHvb1H3hCuHLPfKJ98smA",
            },
            "toTokenMint": "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
          },
        ],
      }
    `);
  });

  test(`restaking.fragJTO.reward.resolve`, async () => {
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
        "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
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
