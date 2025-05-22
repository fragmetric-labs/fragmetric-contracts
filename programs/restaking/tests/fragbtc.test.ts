import type { restakingTypes } from '@fragmetric-labs/sdk';
import { isSome } from '@solana/kit';
import { afterAll, beforeAll, describe, expect, test } from 'vitest';
import { createTestSuiteContext, expectMasked } from '../../testutil';
import { initializeFragBTC } from './fragbtc.init';

/** Scenario Flow
 * 1. user1 deposit 1 zBTC
 * 2. user1 deposit 1 cbBTC
 * 3. user2 deposit 1 wBTC (+1 slot)
 * 4. airdrop 1000 ZEUS reward to Solv vault
 * 5. run operator cycle -> do settle
 *  5-1. check user1 reward account status
 * 6. user1 updateRewardPools
 *  6-1. check user1 reward account status
 *  6-2. check user2 reward account status
 * 7. user2 updateRewardPools
 *  7-1. check user2 reward account status
 * 8. user1 claim total 1
 * 9. airdrop 1000 ZEUS reward to reward_reserve_account
 *  9-1. check global reward account status
 *  9-2. check user1 reward account status
 * 10. global reward settle
 *  10-1. check global reward account status
 * 11. user1 claim total 2
 *  11-1. check user1 reward account status
 * 12. user2 claim total 1
 *  12-1. check user2 reward account status
 * 13. user1 deposit each 1.23456789 * 3 zBTC, cbBTC, wBTC
 * 14. user1 requestWithdrawal each 0.23456789 * 2 zBTC, cbBTC
 * 15. run enqueuWithdrawalBatch, processWithdrawalBatch command
 * 16. user1 withdraw from 1,2 requestIds
 * 17. user1 deposit 1~5 zBTC, cbBTC each
 * 18. delegate user2 reward account -> user1
 * 19. delegate user2 reward account -> user2
 * 20. settle with threshold
 *  20-1. settle not occurs by min amount threshold
 *  20-2. settle not occurs by timestamp threshold
 *  20-3. settle not fully occurs by max amount threshold
 *  20-4. settle occurs by all threshold conditions passed
 */

describe('restaking.fragBTC test', async () => {
  const testCtx = initializeFragBTC(await createTestSuiteContext());

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  const { validator, feePayer, restaking, solv, initializationTasks } = testCtx;
  const ctx = restaking.fragBTC;

  /* create users **/
  const [signer1, signer2] = await Promise.all([
    validator
      .newSigner('fragBTCTestSigner1', 100_000_000_000n)
      .then(async (signer) => {
        await Promise.all([
          validator.airdropToken(
            signer.address,
            'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
            100_000_000_000n
          ),
          validator.airdropToken(
            signer.address,
            'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
            100_000_000_000n
          ),
          validator.airdropToken(
            signer.address,
            '3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh',
            100_000_000_000n
          ),
        ]);
        return signer;
      }),
    validator
      .newSigner('fragBTCTestSigner2', 100_000_000_000n)
      .then(async (signer) => {
        await Promise.all([
          validator.airdropToken(
            signer.address,
            'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
            100_000_000_000n
          ),
          validator.airdropToken(
            signer.address,
            'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
            100_000_000_000n
          ),
          validator.airdropToken(
            signer.address,
            '3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh',
            100_000_000_000n
          ),
        ]);
        return signer;
      }),
  ]);
  const user1 = ctx.user(signer1);
  const user2 = ctx.user(signer2);

  /** 1. configuration test **/
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
          {
            "address": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
            "role": 0,
          },
          {
            "address": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
            "role": 0,
          },
          {
            "address": "E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B",
            "role": 0,
          },
        ],
        "depositResidualMicroReceiptTokenAmount": 0n,
        "metadata": null,
        "normalizedToken": null,
        "oneReceiptTokenAsSOL": 0n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "receiptTokenSupply": 0n,
        "restakingVaultReceiptTokens": [
          {
            "mint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "vault": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
          },
          {
            "mint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "vault": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
          },
          {
            "mint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "vault": "E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B",
          },
        ],
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
            "withdrawalResidualMicroAssetAmount": 0n,
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
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 647695086539n,
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
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 0n,
            "tokenAccumulatedDepositAmount": 0n,
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
            "distributingRewardTokens": [],
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
            "distributingRewardTokens": [],
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

  test(`restaking.fragBTC.reward.resolve`, async () => {
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
                "decimals": 6,
                "description": "ZEUS incentive",
                "id": 1,
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
          {
            "claimable": true,
            "decimals": 6,
            "description": "ZEUS incentive",
            "id": 1,
            "mint": "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
            "name": "ZEUS",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          },
        ],
      }
    `);
  });

  /** 2. deposit test **/
  test('user can deposit token with OrcaDEXLiquidityPool pricing source', async () => {
    await expectMasked(
      user1.deposit.execute(
        {
          assetMint: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
          assetAmount: 100_000_000n,
        },
        { signers: [signer1] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": 100000000n,
          "assetMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
          "metadata": null,
        },
        "events": {
          "unknown": [],
          "userCreatedOrUpdatedFundAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
            "userFundAccount": "DdVfA4rT4tSJRMi688zb5SeZitH91AcKXU79Q4A3pCHg",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
            "userRewardAccount": "8XYL5QhcGSVZFX1wCdvVatXhehd7fwx9CYY4og1Fobt9",
          },
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 100000000n,
            "fundAccount": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
            "mintedReceiptTokenAmount": 100000000n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            },
            "updatedUserRewardAccounts": [
              "8XYL5QhcGSVZFX1wCdvVatXhehd7fwx9CYY4og1Fobt9",
            ],
            "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
            "userFundAccount": "DdVfA4rT4tSJRMi688zb5SeZitH91AcKXU79Q4A3pCHg",
            "userReceiptTokenAccount": "31GDTcPyFTexmiwSvLTZAcY2JSxafoy3yVavK4iAYaLE",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "CHBjDuyN1JxAfeYY7wz68b78E6uPpek1TKqjKeyDJZD6",
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
    ).resolves.toEqual(100_000_000n);

    await expect(user1.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "lamports": 99962596960n,
        "maxWithdrawalRequests": 4,
        "receiptTokenAmount": 100000000n,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "supportedAssets": [
          {
            "amount": 99900000000n,
            "decimals": 8,
            "depositable": true,
            "mint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
          },
          {
            "amount": 100000000000n,
            "decimals": 8,
            "depositable": true,
            "mint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
          },
          {
            "amount": 100000000000n,
            "decimals": 8,
            "depositable": true,
            "mint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
          },
        ],
        "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
        "withdrawalRequests": [],
        "wrappedTokenAmount": 0n,
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);

    await expect(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "G45gQa12Uwvnrp2Yb9oWTSwZSEHZWL71QDWvyLz23bNc",
        "__pricingSources": [
          {
            "address": "4yp9YAXCJsKWMDZq2Q4j4amktvJGXBCpr3Lmv2cYBrb8",
            "role": 0,
          },
          {
            "address": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
            "role": 0,
          },
          {
            "address": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
            "role": 0,
          },
          {
            "address": "E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B",
            "role": 0,
          },
        ],
        "depositResidualMicroReceiptTokenAmount": 0n,
        "metadata": null,
        "normalizedToken": null,
        "oneReceiptTokenAsSOL": 647695086539n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "receiptTokenSupply": 100000000n,
        "restakingVaultReceiptTokens": [
          {
            "mint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "vault": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
          },
          {
            "mint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "vault": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
          },
          {
            "mint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "vault": "E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 8,
            "depositable": true,
            "mint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            "oneTokenAsReceiptToken": 100000000n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 100000000n,
            "operationTotalAmount": 100000000n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 100000000n,
            "withdrawalLastBatchProcessedAt": 1970-01-01T00:00:00.000Z,
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "oneTokenAsReceiptToken": 100000000n,
            "oneTokenAsSol": 647695086539n,
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
          {
            "decimals": 8,
            "depositable": true,
            "mint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
            "oneTokenAsReceiptToken": 100000000n,
            "oneTokenAsSol": 647695086539n,
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
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);

    await expectMasked(user1.reward.resolve(true)).resolves
      .toMatchInlineSnapshot(`
      {
        "basePool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "settlements": [
            {
              "claimedAmount": 0n,
              "reward": {
                "claimable": true,
                "decimals": 6,
                "description": "ZEUS incentive",
                "id": 1,
                "mint": "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
                "name": "ZEUS",
                "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
              },
              "settledAmount": 0n,
              "settledContribution": "MASKED(/[.*C|c]ontribution?$/)",
              "settledSlot": "MASKED(/[.*S|s]lots?$/)",
            },
          ],
          "tokenAllocatedAmount": {
            "records": [
              {
                "amount": 100000000n,
                "contributionAccrualRate": 1,
              },
            ],
            "totalAmount": 100000000n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "bonusPool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "settlements": [
            {
              "claimedAmount": 0n,
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
              "settledContribution": "MASKED(/[.*C|c]ontribution?$/)",
              "settledSlot": "MASKED(/[.*S|s]lots?$/)",
            },
          ],
          "tokenAllocatedAmount": {
            "records": [
              {
                "amount": 100000000n,
                "contributionAccrualRate": 1,
              },
            ],
            "totalAmount": 100000000n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "delegate": null,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
      }
    `);
  });

  test('user can deposit token with PeggedToken pricing source', async () => {
    await expectMasked(
      user1.deposit.execute(
        {
          assetMint: 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
          assetAmount: 100_000_000n,
        },
        { signers: [signer1] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": 100000000n,
          "assetMint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
          "metadata": null,
        },
        "events": {
          "unknown": [],
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 100000000n,
            "fundAccount": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
            "mintedReceiptTokenAmount": 100000000n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            },
            "updatedUserRewardAccounts": [
              "8XYL5QhcGSVZFX1wCdvVatXhehd7fwx9CYY4og1Fobt9",
            ],
            "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
            "userFundAccount": "DdVfA4rT4tSJRMi688zb5SeZitH91AcKXU79Q4A3pCHg",
            "userReceiptTokenAccount": "31GDTcPyFTexmiwSvLTZAcY2JSxafoy3yVavK4iAYaLE",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "8jVRhbnfgoanH33eUsJcWgvuaPrSqynGr87hv9C3zpSy",
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
    ).resolves.toEqual(200_000_000n);

    await expect(user1.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "lamports": 99962596960n,
        "maxWithdrawalRequests": 4,
        "receiptTokenAmount": 200000000n,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "supportedAssets": [
          {
            "amount": 99900000000n,
            "decimals": 8,
            "depositable": true,
            "mint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
          },
          {
            "amount": 99900000000n,
            "decimals": 8,
            "depositable": true,
            "mint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
          },
          {
            "amount": 100000000000n,
            "decimals": 8,
            "depositable": true,
            "mint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
          },
        ],
        "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
        "withdrawalRequests": [],
        "wrappedTokenAmount": 0n,
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);

    await expectMasked(user1.reward.resolve(true)).resolves
      .toMatchInlineSnapshot(`
      {
        "basePool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "settlements": [
            {
              "claimedAmount": 0n,
              "reward": {
                "claimable": true,
                "decimals": 6,
                "description": "ZEUS incentive",
                "id": 1,
                "mint": "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
                "name": "ZEUS",
                "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
              },
              "settledAmount": 0n,
              "settledContribution": "MASKED(/[.*C|c]ontribution?$/)",
              "settledSlot": "MASKED(/[.*S|s]lots?$/)",
            },
          ],
          "tokenAllocatedAmount": {
            "records": [
              {
                "amount": 200000000n,
                "contributionAccrualRate": 1,
              },
            ],
            "totalAmount": 200000000n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "bonusPool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "settlements": [
            {
              "claimedAmount": 0n,
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
              "settledContribution": "MASKED(/[.*C|c]ontribution?$/)",
              "settledSlot": "MASKED(/[.*S|s]lots?$/)",
            },
          ],
          "tokenAllocatedAmount": {
            "records": [
              {
                "amount": 200000000n,
                "contributionAccrualRate": 1,
              },
            ],
            "totalAmount": 200000000n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "delegate": null,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
      }
    `);

    await validator.skipSlots(1n);
    await expectMasked(
      user2.deposit.execute(
        {
          assetMint: '3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh',
          assetAmount: 100_000_000n,
        },
        { signers: [signer2] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": 100000000n,
          "assetMint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
          "metadata": null,
        },
        "events": {
          "unknown": [],
          "userCreatedOrUpdatedFundAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "user": "CCgWk6XBuLqZTcDuJa4ZKGUM22cLXUFAxJSHiZdm3T3b",
            "userFundAccount": "4qyutWBhvuM6k73kehvo6drqQg93aLJaEiPo4ticeQE1",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "user": "CCgWk6XBuLqZTcDuJa4ZKGUM22cLXUFAxJSHiZdm3T3b",
            "userRewardAccount": "5vjNrtqwKRB68nTfwN3gZayEADap2947MuZp43bCdLo1",
          },
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 100000000n,
            "fundAccount": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
            "mintedReceiptTokenAmount": 100000000n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
            },
            "updatedUserRewardAccounts": [
              "5vjNrtqwKRB68nTfwN3gZayEADap2947MuZp43bCdLo1",
            ],
            "user": "CCgWk6XBuLqZTcDuJa4ZKGUM22cLXUFAxJSHiZdm3T3b",
            "userFundAccount": "4qyutWBhvuM6k73kehvo6drqQg93aLJaEiPo4ticeQE1",
            "userReceiptTokenAccount": "HJVHLQtg6MJpTtyzbTCEzdLxLF6XShRvT7gcYbBUwdzD",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "ALt5RC3MSK2K9DNk7Ad9pk5vuCzStXKywHa6M1sZGVmt",
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
      user2.receiptToken.resolve(true).then((res) => res?.amount)
    ).resolves.toEqual(100_000_000n);

    await expectMasked(user2.reward.resolve(true)).resolves
      .toMatchInlineSnapshot(`
      {
        "basePool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "settlements": [
            {
              "claimedAmount": 0n,
              "reward": {
                "claimable": true,
                "decimals": 6,
                "description": "ZEUS incentive",
                "id": 1,
                "mint": "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
                "name": "ZEUS",
                "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
              },
              "settledAmount": 0n,
              "settledContribution": "MASKED(/[.*C|c]ontribution?$/)",
              "settledSlot": "MASKED(/[.*S|s]lots?$/)",
            },
          ],
          "tokenAllocatedAmount": {
            "records": [
              {
                "amount": 100000000n,
                "contributionAccrualRate": 1,
              },
            ],
            "totalAmount": 100000000n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "bonusPool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "settlements": [
            {
              "claimedAmount": 0n,
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
              "settledContribution": "MASKED(/[.*C|c]ontribution?$/)",
              "settledSlot": "MASKED(/[.*S|s]lots?$/)",
            },
          ],
          "tokenAllocatedAmount": {
            "records": [
              {
                "amount": 100000000n,
                "contributionAccrualRate": 1,
              },
            ],
            "totalAmount": 100000000n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "delegate": null,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "user": "CCgWk6XBuLqZTcDuJa4ZKGUM22cLXUFAxJSHiZdm3T3b",
      }
    `);

    await expect(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "G45gQa12Uwvnrp2Yb9oWTSwZSEHZWL71QDWvyLz23bNc",
        "__pricingSources": [
          {
            "address": "4yp9YAXCJsKWMDZq2Q4j4amktvJGXBCpr3Lmv2cYBrb8",
            "role": 0,
          },
          {
            "address": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
            "role": 0,
          },
          {
            "address": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
            "role": 0,
          },
          {
            "address": "E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B",
            "role": 0,
          },
        ],
        "depositResidualMicroReceiptTokenAmount": 0n,
        "metadata": null,
        "normalizedToken": null,
        "oneReceiptTokenAsSOL": 647695086539n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "receiptTokenSupply": 300000000n,
        "restakingVaultReceiptTokens": [
          {
            "mint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "vault": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
          },
          {
            "mint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "vault": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
          },
          {
            "mint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "vault": "E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 8,
            "depositable": true,
            "mint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            "oneTokenAsReceiptToken": 100000000n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 100000000n,
            "operationTotalAmount": 100000000n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 100000000n,
            "withdrawalLastBatchProcessedAt": 1970-01-01T00:00:00.000Z,
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "oneTokenAsReceiptToken": 100000000n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 100000000n,
            "operationTotalAmount": 100000000n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 100000000n,
            "withdrawalLastBatchProcessedAt": 1970-01-01T00:00:00.000Z,
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
            "oneTokenAsReceiptToken": 100000000n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 100000000n,
            "operationTotalAmount": 100000000n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 100000000n,
            "withdrawalLastBatchProcessedAt": 1970-01-01T00:00:00.000Z,
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);
  });

  test('user can update reward pools and sync with global reward account anytime', async () => {
    const user1Reward_1 = await user1.reward.resolve(true);

    // user1 updateRewardPools
    await user1.reward.updatePools.execute(null);

    const user1Reward_2 = await user1.reward.resolve(true);

    const elapsedSlots =
      user1Reward_2?.basePool.updatedSlot! -
      user1Reward_1?.basePool.updatedSlot!;
    const increasedContribution =
      user1Reward_2?.basePool.contribution! -
      user1Reward_1?.basePool.contribution!;
    expect(increasedContribution, 't7_1').toEqual(
      elapsedSlots *
        user1Reward_2?.basePool.tokenAllocatedAmount.records[0].amount! *
        BigInt(
          user1Reward_2?.basePool.tokenAllocatedAmount.records[0]
            .contributionAccrualRate!
        ) *
        100n
    );
    expect(
      user1Reward_2?.basePool.settlements[0].settledAmount,
      't7_2'
    ).toEqual(0n);
  });

  /** 3. operation and reward **/
  test('fund can settle distributing rewards by operation', async () => {
    // drop distribution token to the vault
    await validator.airdropToken(
      solv.zBTC.address!,
      'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
      100_000_000_000n
    );

    // run operator to harvest
    await ctx.fund.runCommand.executeChained(null);

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

    const globalReward_1 = await ctx.reward.resolve(true);
    await expectMasked(globalReward_1).resolves.toMatchInlineSnapshot(`
      {
        "basePool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "customContributionAccrualRateEnabled": false,
          "initialSlot": "MASKED(/[.*S|s]lots?$/)",
          "settlements": [
            {
              "blocks": [
                {
                  "amount": 100000000000n,
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
                "description": "ZEUS incentive",
                "id": 1,
                "mint": "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
                "name": "ZEUS",
                "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
              },
              "settledAmount": 100000000000n,
              "settlementBlocksLastRewardPoolContribution": "MASKED(/[.*C|c]ontribution?$/)",
              "settlementBlocksLastSlot": "MASKED(/[.*S|s]lots?$/)",
            },
          ],
          "tokenAllocatedAmount": {
            "records": [
              {
                "amount": 300000000n,
                "contributionAccrualRate": 1,
              },
            ],
            "totalAmount": 300000000n,
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
            "records": [
              {
                "amount": 300000000n,
                "contributionAccrualRate": 1,
              },
            ],
            "totalAmount": 300000000n,
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
          {
            "claimable": true,
            "decimals": 6,
            "description": "ZEUS incentive",
            "id": 1,
            "mint": "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
            "name": "ZEUS",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          },
        ],
      }
    `);
    const globalSettledSlot_1 =
      globalReward_1?.basePool.settlements[0].blocks[0].endingSlot!;

    const settlementBlockContributions = globalReward_1?.basePool.settlements
      .map((settlement, i) =>
        settlement.blocks.map((block, j) => {
          const blockContribution =
            block.endingContribution - block.startingContribution;
          return blockContribution;
        })
      )
      .flat();

    // check user reward accounts
    const user1Reward_2 = await user1.reward.resolve(true);
    await expectMasked(user1Reward_2).resolves.toMatchInlineSnapshot(`
      {
        "basePool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "settlements": [
            {
              "claimedAmount": 0n,
              "reward": {
                "claimable": true,
                "decimals": 6,
                "description": "ZEUS incentive",
                "id": 1,
                "mint": "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
                "name": "ZEUS",
                "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
              },
              "settledAmount": 0n,
              "settledContribution": "MASKED(/[.*C|c]ontribution?$/)",
              "settledSlot": "MASKED(/[.*S|s]lots?$/)",
            },
          ],
          "tokenAllocatedAmount": {
            "records": [
              {
                "amount": 200000000n,
                "contributionAccrualRate": 1,
              },
            ],
            "totalAmount": 200000000n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "bonusPool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "settlements": [
            {
              "claimedAmount": 0n,
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
              "settledContribution": "MASKED(/[.*C|c]ontribution?$/)",
              "settledSlot": "MASKED(/[.*S|s]lots?$/)",
            },
          ],
          "tokenAllocatedAmount": {
            "records": [
              {
                "amount": 200000000n,
                "contributionAccrualRate": 1,
              },
            ],
            "totalAmount": 200000000n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "delegate": null,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
      }
    `);

    await user1.reward.updatePools.execute(null);
    const user1Reward_3 = await user1.reward.resolve(true);

    expect(
      user1Reward_3?.basePool.settlements[0].settledAmount,
      't8_1'
    ).toBeGreaterThan(0n);
    expect(user1Reward_3?.basePool.settlements[0].reward.mint, 't8_2').toEqual(
      'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq'
    );

    const user1ElapsedSettledSlots_1 =
      globalSettledSlot_1 - user1Reward_2?.basePool.updatedSlot!;
    const user1SettledContribution_1 =
      user1ElapsedSettledSlots_1 *
        user1Reward_2?.basePool.tokenAllocatedAmount.records[0].amount! *
        BigInt(
          user1Reward_2?.basePool.tokenAllocatedAmount.records[0]
            .contributionAccrualRate!
        ) *
        100n +
      user1Reward_2?.basePool.contribution!;
    expect(
      user1Reward_3?.basePool.settlements[0].settledContribution,
      't8_3'
    ).toEqual(user1SettledContribution_1);

    // this is real formula at onchain
    // const maxSlot_1 =
    //   user1Reward_2?.basePool.settlements[0].settledSlot! <
    //   user1Reward_2?.basePool.updatedSlot!
    //     ? user1Reward_2?.basePool.updatedSlot
    //     : user1Reward_2?.basePool.settlements[0].settledSlot;
    // const user1SettledContribution_1 =
    //   user1Reward_2?.basePool.contribution! -
    //   user1Reward_2?.basePool.settlements[0]
    //     .settledContribution! +
    //   (globalReward_1?.basePool.settlements[0].blocks[0].endingSlot! - maxSlot_1!) *
    //     user1Reward_2?.basePool.tokenAllocatedAmount
    //       .records[0].amount! *
    //     BigInt(user1Reward_2?.basePool.tokenAllocatedAmount.records[0].contributionAccrualRate!) *
    //     100n;
    // console.log(
    //   `[user1] maxSlot: ${maxSlot_1}, last settled slot: ${user1Reward_2?.basePool.settlements[0].settledSlot}, last updated slot: ${user1Reward_2?.basePool.updatedSlot}, calculated settled contribution: ${user1SettledContribution_1}, real settled contribution: ${user1Reward_3?.basePool.settlements[0].settledContribution}`
    // );

    const user1SettledAmount =
      (user1SettledContribution_1 *
        globalReward_1?.basePool.settlements[0].settledAmount!) /
      settlementBlockContributions![0];

    expect(
      user1Reward_3?.basePool.settlements[0].settledAmount,
      't8_4'
    ).toEqual(user1SettledAmount);

    const user2Reward_1 = await user2.reward.resolve(true);
    await expectMasked(user2Reward_1).resolves.toMatchInlineSnapshot(`
      {
        "basePool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "settlements": [
            {
              "claimedAmount": 0n,
              "reward": {
                "claimable": true,
                "decimals": 6,
                "description": "ZEUS incentive",
                "id": 1,
                "mint": "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
                "name": "ZEUS",
                "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
              },
              "settledAmount": 0n,
              "settledContribution": "MASKED(/[.*C|c]ontribution?$/)",
              "settledSlot": "MASKED(/[.*S|s]lots?$/)",
            },
          ],
          "tokenAllocatedAmount": {
            "records": [
              {
                "amount": 100000000n,
                "contributionAccrualRate": 1,
              },
            ],
            "totalAmount": 100000000n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "bonusPool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "settlements": [
            {
              "claimedAmount": 0n,
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
              "settledContribution": "MASKED(/[.*C|c]ontribution?$/)",
              "settledSlot": "MASKED(/[.*S|s]lots?$/)",
            },
          ],
          "tokenAllocatedAmount": {
            "records": [
              {
                "amount": 100000000n,
                "contributionAccrualRate": 1,
              },
            ],
            "totalAmount": 100000000n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "delegate": null,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "user": "CCgWk6XBuLqZTcDuJa4ZKGUM22cLXUFAxJSHiZdm3T3b",
      }
    `);

    await user2.reward.updatePools.execute(null);
    const user2Reward_2 = await user2.reward.resolve(true);
  });

  /** 4. claim */
  test('user can claim reward token after settlement', async () => {
    const claimRes = await user1.reward.claim.execute(
      {
        mint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
        amount: null,
        recipient: null,
      },
      { signers: [signer1] }
    );

    const claimedRewardAmount =
      claimRes.events?.userClaimedReward?.claimedRewardTokenAmount;

    // check claimed amount at settlement equals to real claimed amount from claim event
    const user1Reward_4 = await user1.reward.resolve(true);
    expect(
      user1Reward_4?.basePool.settlements.map(
        (settlement) => settlement.claimedAmount
      ),
      't9_1'
    ).toEqual([claimedRewardAmount]);

    // check claimed amount has moved to user1's reward token account
    await expect(
      user1.rewardTokens
        .resolve(true)
        .then((tokens) => tokens.map((token) => token?.amount)),
      't9_2'
    ).resolves.toEqual([undefined, claimedRewardAmount]);

    const user1RewardTokenAccounts = await user1.rewardTokens.resolve(true);
    const user1ZeusRewardTokenAccount = user1RewardTokenAccounts.filter(
      (tokenAccount) =>
        tokenAccount !== null &&
        tokenAccount.mint.toString() ==
          'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq'
    )[0];
    expect(user1ZeusRewardTokenAccount.amount, 't9_3').toEqual(
      claimedRewardAmount
    );

    const globalReward_2 = await ctx.reward.resolve(true);
    expect(
      globalReward_2?.basePool.settlements[0].claimedAmount,
      't9_4'
    ).toEqual(user1Reward_4?.basePool.settlements[0].claimedAmount);
  });

  test('settle -> everyone claim -> update reward pool -> check the remaining amount', async () => {
    // drop distribution token to the vault
    const rewardAmount = 100_000_000_000n;
    await validator.airdropToken(
      // reward_reserve_account to only test about settle
      (await ctx.reward.reserve.resolveAddress())!, // directly drop reward at fragBTC reward reserve account
      'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
      rewardAmount
    );

    const globalReward_2 = await ctx.reward.resolve(true);
    const user1Reward_4 = await user1.reward.resolve(true);
    const user2Reward_2 = await user2.reward.resolve(true);

    // settle global reward
    const settleRewardRes = await ctx.reward.settleReward.execute({
      mint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
      amount: rewardAmount,
    });
    await expectMasked(settleRewardRes).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "amount": 100000000000n,
          "isBonus": false,
          "mint": "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
        },
        "events": {
          "fundManagerUpdatedRewardPool": {
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "rewardAccount": "DL2iRqSqzP4jDdvoZNd2kqvKF17EyKoKXHaeUCGtLKoE",
          },
          "unknown": [],
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);

    // check global reward pool settlement amount delta
    const globalReward_3 = await ctx.reward.resolve(true);

    const settledAmountDeltas = globalReward_3?.basePool.settlements.map(
      (afterSettlement, i) => {
        const beforeSettlement = globalReward_2?.basePool.settlements[i];
        const beforeSettledAmount = beforeSettlement?.settledAmount ?? 0n;
        return afterSettlement.settledAmount - beforeSettledAmount;
      }
    );
    expect(settledAmountDeltas, 't10_1').toEqual([rewardAmount]);

    await user1.reward.updatePools.execute(null); // just did updatedPools to check abount claimedRewardTokenAmount at the event is correct
    const user1Reward_5 = await user1.reward.resolve(true);

    // user1 claim total
    const user1ClaimRes = await user1.reward.claim.execute(
      {
        mint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
        amount: null,
        recipient: null,
      },
      { signers: [signer1] }
    );

    // check user1 reward pool settlement amount delta
    const user1Reward_6 = await user1.reward.resolve(true);

    const globalSettledContribution_1 =
      globalReward_3?.basePool.settlements[0].blocks[0].endingContribution! -
      globalReward_3?.basePool.settlements[0].blocks[0].startingContribution!;

    expect(
      user1ClaimRes.events?.userClaimedReward?.claimedRewardTokenAmount,
      't10_2'
    ).toEqual(
      user1Reward_5?.basePool.settlements[0].settledAmount! -
        user1Reward_5?.basePool.settlements[0].claimedAmount!
    );

    const user1SettledAmountDeltas = user1Reward_6?.basePool.settlements.map(
      (afterSettlement, i) => {
        const beforeSettlement = user1Reward_4?.basePool.settlements[i];
        const beforeSettledAmount = beforeSettlement?.settledAmount ?? 0n;
        return afterSettlement.settledAmount - beforeSettledAmount;
      }
    );
    expect(user1SettledAmountDeltas!, 't10_3').toEqual([
      (rewardAmount *
        (user1Reward_6?.basePool.settlements[0].settledContribution! -
          user1Reward_4?.basePool.settlements[0].settledContribution!)) /
        globalSettledContribution_1,
    ]);

    // user2 claim total
    await user2.reward.claim.execute(
      {
        mint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
        amount: null,
        recipient: null,
      },
      { signers: [signer2] }
    );

    const user2Reward_3 = await user2.reward.resolve(true);

    const user2SettledAmountDeltas = user2Reward_3?.basePool.settlements.map(
      (afterSettlement, i) => {
        const beforeSettlement = user2Reward_2?.basePool.settlements[i];
        const beforeSettledAmount = beforeSettlement?.settledAmount ?? 0n;
        return afterSettlement.settledAmount - beforeSettledAmount;
      }
    );
    expect(user2SettledAmountDeltas!, ' t10_4').toEqual([
      (rewardAmount *
        (user2Reward_3?.basePool.settlements[0].settledContribution! -
          user2Reward_2?.basePool.settlements[0].settledContribution!)) /
        globalSettledContribution_1,
    ]);

    // update global reward
    await validator.skipSlots(2n);
    await ctx.reward.updatePools.execute(null);

    // check remaing amount == 1
    const globalReward_4 = await ctx.reward.resolve(true);

    expect(globalReward_4?.basePool.settlements[0].claimedAmount).toEqual(
      user1Reward_6?.basePool.settlements[0].claimedAmount! +
        user2Reward_3?.basePool.settlements[0].claimedAmount!
    );
  });

  /** 5. withdraw and pegging **/
  test('funds supporting only a single token and tokens pegged to it must issue receipt tokens at a 1:1 ratio until additional yield is compounded', async () => {
    let expectedReceiptTokenSupply = await ctx
      .resolve(true)
      .then((data) => data!.receiptTokenSupply);

    for (let i = 1; i <= 9; i++) {
      const assetMint = [
        'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
        'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
        '3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh',
      ][i % 3];
      const assetAmount = 123_456_789n * BigInt(i);
      expectedReceiptTokenSupply += assetAmount;

      await expect(
        user1.deposit.execute(
          {
            assetMint,
            assetAmount,
          },
          { signers: [signer1] }
        )
      ).resolves.toMatchObject({
        events: {
          userDepositedToFund: {
            depositedAmount: assetAmount,
            mintedReceiptTokenAmount: assetAmount,
          },
        },
      });
    }

    await expect(
      ctx.resolve(true),
      'receiptTokenSupply should be exactly increased as much as newly deposited asset amount'
    ).resolves.toMatchObject({
      receiptTokenSupply: expectedReceiptTokenSupply,
    });

    for (let i = 1; i <= 4; i++) {
      const receiptTokenAmount = 23_456_789n * BigInt(i);
      const assetMint = [
        'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
        'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
      ][i % 2];
      expectedReceiptTokenSupply -= receiptTokenAmount;

      await expect(
        user1.requestWithdrawal.execute(
          {
            assetMint: assetMint,
            receiptTokenAmount: receiptTokenAmount,
          },
          { signers: [signer1] }
        )
      ).resolves.toMatchObject({
        events: {
          userRequestedWithdrawalFromFund: {
            supportedTokenMint: { value: assetMint },
            requestedReceiptTokenAmount: receiptTokenAmount,
          },
        },
      });
    }
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'EnqueueWithdrawalBatch',
    });
    const res = await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'ProcessWithdrawalBatch',
    });
    const evt = res.events!.operatorRanFundCommand!;
    const result = isSome(evt.result)
      ? (evt.result.value
          .fields[0] as restakingTypes.ProcessWithdrawalBatchCommandResult)
      : null;
    expect(result!.requestedReceiptTokenAmount).toEqual(
      result!.processedReceiptTokenAmount
    );
    expect(
      result!.reservedAssetUserAmount + result!.deductedAssetFeeAmount,
      'reduced asset amount must be equal to burnt receipt token amount'
    ).toEqual(result!.processedReceiptTokenAmount);

    for (let assetMint of [
      'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
      'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
    ]) {
      for (let i = 1; i <= 2; i++) {
        const res = await user1.withdraw.execute(
          {
            assetMint,
            requestId: BigInt(i),
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
      }
    }

    await expect(
      ctx.resolve(true),
      'receipt token supply reduced as withdrawal reqs being processed'
    ).resolves.toMatchObject({
      receiptTokenSupply: expectedReceiptTokenSupply,
    });

    let withdrawalResidualMicroAssetAmount = 0n;
    await expect(
      ctx.resolve(true).then((data) => {
        withdrawalResidualMicroAssetAmount =
          data!.supportedAssets.reduce((sum, asset) => {
            return sum + asset.withdrawalResidualMicroAssetAmount;
          }, 0n) / 1_000_000n;
        return (
          data!.supportedAssets.reduce((sum, asset) => {
            return sum + asset.operationTotalAmount;
          }, 0n) - withdrawalResidualMicroAssetAmount
        );
      }),
      'sum of all underlying assets must be equal to receipt token supply (fund accounting)'
    ).resolves.toEqual(expectedReceiptTokenSupply);

    await expect(
      ctx.fund.reserve.supportedTokens
        .resolve(true)
        .then((accounts) =>
          accounts.reduce(
            (sum, tokenAccount) =>
              tokenAccount ? sum + tokenAccount.amount : sum,
            -withdrawalResidualMicroAssetAmount
          )
        ),
      'sum of all assets must be equal to receipt token supply (token account)'
    ).resolves.toEqual(expectedReceiptTokenSupply);

    for (let i = 1; i <= 5; i++) {
      const assetMint =
        i % 2 == 0
          ? 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg'
          : 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij';
      const assetAmount = BigInt(i);
      expectedReceiptTokenSupply += assetAmount;

      const res = await user1.deposit.execute(
        {
          assetMint,
          assetAmount,
        },
        { signers: [signer1] }
      );
      const evt = res.events!.userDepositedToFund!;
      expect(
        evt.mintedReceiptTokenAmount,
        'mintedReceiptTokenAmount = assetAmount - [optional residual error up to one denomalized unit]'
      ).toBeOneOf([assetAmount, assetAmount - 1n]);
    }

    await expect(
      ctx.resolve(true).then((data) => data?.receiptTokenSupply)
    ).resolves.toBeOneOf([
      expectedReceiptTokenSupply,
      expectedReceiptTokenSupply - 1n,
    ]);
  });

  /** 6. delegate */
  test('delegate reward account from user2 to user1', async () => {
    const user2DelegateRes = await user2.reward.delegate.execute(
      { newDelegate: signer1.address },
      { signers: [signer2] }
    );
    await expectMasked(user2DelegateRes).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "delegate": null,
          "newDelegate": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
        },
        "events": {
          "unknown": [],
          "userDelegatedRewardAccount": {
            "delegate": {
              "__option": "Some",
              "value": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
            },
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "user": "CCgWk6XBuLqZTcDuJa4ZKGUM22cLXUFAxJSHiZdm3T3b",
            "userRewardAccount": "5vjNrtqwKRB68nTfwN3gZayEADap2947MuZp43bCdLo1",
          },
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);
    await expect(
      user2.reward.resolve(true).then((res) => res?.delegate)
    ).resolves.toEqual(signer1.address);

    await user2.reward.delegate.execute(
      { delegate: signer1.address, newDelegate: signer2.address },
      { signers: [signer1] }
    );

    // fails to delegate
    await expect(
      user2.reward.delegate.execute(
        { delegate: signer1.address, newDelegate: signer2.address },
        { signers: [signer1] }
      )
    ).rejects.toThrowError('Transaction simulation failed'); // reward: user reward account authority must be either user or delegate
  });

  /** 7. settle with threshold */
  test('settle should not occur if threshold is not matched', async () => {
    const rewardTokenMint = 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq';

    // 1. update distributing reward amount threshold -> settle would not occur
    await ctx.fund.updateRestakingVaultDistributingRewardHarvestThreshold.execute(
      {
        vault: solv.zBTC.address!,
        rewardTokenMint,
        harvestThresholdMinAmount: 600_000_000n,
        harvestThresholdMaxAmount: 700_000_000n,
        harvestThresholdIntervalSeconds: 1n,
      }
    );

    // drop distribution token to the vault
    await validator.airdropToken(
      solv.zBTC.address!,
      rewardTokenMint,
      500_000_000n
    );

    const globalReward_4 = await ctx.reward.resolve(true);

    // try to settle global reward
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestReward',
      operator: restaking.knownAddresses.fundManager,
    });

    const globalReward_5 = await ctx.reward.resolve(true);

    // settle not occurs
    expect(
      globalReward_5?.basePool.settlements[0].settlementBlocksLastSlot!
    ).toEqual(
      globalReward_4?.basePool.settlements[0].settlementBlocksLastSlot!
    );

    // 2. update distributing reward interval second threshold -> settle would not occur
    await ctx.fund.updateRestakingVaultDistributingRewardHarvestThreshold.execute(
      {
        vault: solv.zBTC.address!,
        rewardTokenMint,
        harvestThresholdMinAmount: 200_000_000n,
        harvestThresholdMaxAmount: 300_000_000n,
        harvestThresholdIntervalSeconds: 100n,
      }
    );

    // try to settle global reward
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestReward',
      operator: restaking.knownAddresses.fundManager,
    });

    const globalReward_6 = await ctx.reward.resolve(true);

    // settle not occurs
    expect(
      globalReward_6?.basePool.settlements[0].settlementBlocksLastSlot!
    ).toEqual(
      globalReward_4?.basePool.settlements[0].settlementBlocksLastSlot!
    );

    // 3. update distributing reward interval second threshold -> now settle would occur but not fully settled due to max amount threshold
    await ctx.fund.updateRestakingVaultDistributingRewardHarvestThreshold.execute(
      {
        vault: solv.zBTC.address!,
        rewardTokenMint,
        harvestThresholdMinAmount: 200_000_000n,
        harvestThresholdMaxAmount: 300_000_000n,
        harvestThresholdIntervalSeconds: 0n,
      }
    );

    // now settle occurs but not fully settled
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestReward',
      operator: restaking.knownAddresses.fundManager,
    });

    const globalReward_7 = await ctx.reward.resolve(true);

    expect(
      globalReward_7?.basePool.settlements[0].settlementBlocksLastSlot!
    ).toBeGreaterThan(
      globalReward_4?.basePool.settlements[0].settlementBlocksLastSlot!
    );

    expect(
      globalReward_7?.basePool.settlements[0].settledAmount! -
        globalReward_4?.basePool.settlements[0].settledAmount!
    ).toEqual(300_000_000n);

    // 4. now fully settled
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestReward',
      operator: restaking.knownAddresses.fundManager,
    });

    const globalReward_8 = await ctx.reward.resolve(true);

    expect(
      globalReward_8?.basePool.settlements[0].settlementBlocksLastSlot!
    ).toBeGreaterThan(
      globalReward_7?.basePool.settlements[0].settlementBlocksLastSlot!
    );

    expect(
      globalReward_8?.basePool.settlements[0].settledAmount! -
        globalReward_7?.basePool.settlements[0].settledAmount!
    ).toEqual(200_000_000n);
  });
});
