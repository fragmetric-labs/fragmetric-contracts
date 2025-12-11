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
 * 18. settle with threshold
 *  18-1. settle not occurs by min amount threshold
 *  18-2. settle not occurs by timestamp threshold
 *  18-3. settle not fully occurs by max amount threshold
 *  18-4. settle occurs by all threshold conditions passed
 * 19. deposit srt to mint rt
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
          validator.airdropToken(
            signer.address,
            'SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8', // srt
            2000_0000_0000n
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
    validator.airdrop(restaking.knownAddresses.fundManager, 100_000_000_000n),
  ]);
  const user1 = ctx.user(signer1);
  const user2 = ctx.user(signer2);

  /** 1. configuration test **/
  test(`restaking.fragBTC initializationTasks snapshot`, async () => {
    await expectMasked(initializationTasks).resolves.toMatchSnapshot();
  });

  test(`restaking.fragBTC.resolve`, async () => {
    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "MASKED(__lookupTableAddress)",
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
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
          },
          {
            "mint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
          },
          {
            "mint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
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
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
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
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
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
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
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
            "tokenWithdrawable": true,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 30,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 0n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "tokenWithdrawable": true,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 30,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 0n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
            "tokenWithdrawable": true,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 30,
          },
        ],
        "generalStrategy": {
          "depositEnabled": true,
          "donationEnabled": false,
          "operationEnabled": true,
          "performanceFeeRateBps": 0,
          "transferEnabled": true,
          "withdrawalBatchThresholdSeconds": 1n,
          "withdrawalEnabled": true,
          "withdrawalFeeRateBps": 20,
        },
        "restakingVaultStrategies": [
          {
            "compoundingRewardTokens": [],
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
            "rewardCommissionRateBps": 0,
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "vault": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
            "vaultReceiptTokenDepositable": true,
          },
          {
            "compoundingRewardTokens": [],
            "delegations": [],
            "distributingRewardTokens": [],
            "pricingSource": {
              "__kind": "SolvBTCVault",
              "address": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
            },
            "rewardCommissionRateBps": 0,
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "vault": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
            "vaultReceiptTokenDepositable": false,
          },
          {
            "compoundingRewardTokens": [],
            "delegations": [],
            "distributingRewardTokens": [],
            "pricingSource": {
              "__kind": "SolvBTCVault",
              "address": "E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B",
            },
            "rewardCommissionRateBps": 0,
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "vault": "E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B",
            "vaultReceiptTokenDepositable": false,
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
          "skipUserFundAccountCreation": false,
          "skipUserRewardAccountCreation": false,
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

    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "MASKED(__lookupTableAddress)",
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
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
          },
          {
            "mint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
          },
          {
            "mint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
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
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 100000000n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
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
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
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
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
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
          "skipUserFundAccountCreation": false,
          "skipUserRewardAccountCreation": false,
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
          "skipUserFundAccountCreation": false,
          "skipUserRewardAccountCreation": false,
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

    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "MASKED(__lookupTableAddress)",
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
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
          },
          {
            "mint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
          },
          {
            "mint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
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
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 100000000n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
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
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 100000000n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
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
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 100000000n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);
  });

  /** 3. reward settlement by operation **/
  test('fund can settle distributing rewards by operation', async () => {
    // drop distribution token to the vault
    await validator.airdropToken(
      solv.zBTC.address!,
      'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
      100_000_000_000n
    );

    // run operator to harvest
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestRestakingYield',
      operator: restaking.knownAddresses.fundManager,
    });

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

  test('settle should not occur if threshold is not matched', async () => {
    const rewardTokenMint = 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq';

    // 1. update distributing reward amount threshold -> settle would not occur
    await ctx.fund.updateRestakingVaultRewardHarvestThreshold.execute({
      vault: solv.zBTC.address!,
      rewardTokenMint,
      harvestThresholdMinAmount: 600_000_000n,
      harvestThresholdMaxAmount: 700_000_000n,
      harvestThresholdIntervalSeconds: 1n,
    });

    // drop distribution token to the vault
    await validator.airdropToken(
      solv.zBTC.address!,
      rewardTokenMint,
      500_000_000n
    );

    const globalReward_4 = await ctx.reward.resolve(true);

    // try to settle global reward
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestRestakingYield',
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
    await ctx.fund.updateRestakingVaultRewardHarvestThreshold.execute({
      vault: solv.zBTC.address!,
      rewardTokenMint,
      harvestThresholdMinAmount: 200_000_000n,
      harvestThresholdMaxAmount: 300_000_000n,
      harvestThresholdIntervalSeconds: 100n,
    });

    // try to settle global reward
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestRestakingYield',
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
    await ctx.fund.updateRestakingVaultRewardHarvestThreshold.execute({
      vault: solv.zBTC.address!,
      rewardTokenMint,
      harvestThresholdMinAmount: 200_000_000n,
      harvestThresholdMaxAmount: 300_000_000n,
      harvestThresholdIntervalSeconds: 0n,
    });

    // now settle occurs but not fully settled
    await validator.skipSlots(1n);
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestRestakingYield',
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
    await validator.skipSlots(1n);
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestRestakingYield',
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

  /** 4. reward claim */
  test('user can claim reward token after settlement', async () => {
    const rewardBefore = (await ctx.reward.resolve(true))!;
    const userRewardBefore = (await user1.reward.resolve(true))!;
    const userRewardAmountBefore = await user1.rewardTokens
      .resolve(true)
      .then(
        (tokens) =>
          tokens.find(
            (token) =>
              token !== null &&
              token.mint == 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq'
          )?.amount ?? 0n
      );

    const res = await user1.reward.claim.execute(
      {
        mint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
        amount: null,
        recipient: null,
      },
      { signers: [signer1] }
    );

    const claimedRewardAmount =
      res.events?.userClaimedReward?.claimedRewardTokenAmount;

    const rewardAfter = (await ctx.reward.resolve(true))!;
    const userRewardAfter = (await user1.reward.resolve(true))!;
    const userRewardAmountAfter = await user1.rewardTokens
      .resolve(true)
      .then(
        (tokens) =>
          tokens.find(
            (token) =>
              token !== null &&
              token.mint == 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq'
          )!.amount
      );

    // check claimed amount at settlement equals to real claimed amount from claim event
    expect(
      rewardAfter.basePool.settlements[0].claimedAmount -
        rewardBefore.basePool.settlements[0].claimedAmount
    ).toEqual(claimedRewardAmount);
    expect(
      userRewardAfter.basePool.settlements[0].claimedAmount -
        userRewardBefore.basePool.settlements[0].claimedAmount
    ).toEqual(claimedRewardAmount);

    // check claimed amount has moved to user1's reward token account
    expect(userRewardAmountAfter - userRewardAmountBefore).toEqual(
      claimedRewardAmount
    );
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

    await ctx.reward.updatePools.execute(null);
    await user1.reward.updatePools.execute(null);
    await user2.reward.updatePools.execute(null);

    const globalReward_0 = (await ctx.reward.resolve(true))!;
    const user1Reward_0 = (await user1.reward.resolve(true))!;
    const user2Reward_0 = (await user2.reward.resolve(true))!;

    // settle global reward
    await expectMasked(
      ctx.reward.settleReward.execute({
        mint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
        amount: rewardAmount,
      })
    ).resolves.toMatchInlineSnapshot(`
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

    const globalReward_1 = (await ctx.reward.resolve(true))!;

    // check global reward pool settlement amount delta
    const globalSettlement_0 = globalReward_0.basePool.settlements[0];
    const globalSettlement_1 = globalReward_1.basePool.settlements[0];
    const lastSettlementBlock_1 =
      globalSettlement_1.blocks[globalSettlement_1.blocks.length - 1];
    const globalSettledContribution_1 =
      lastSettlementBlock_1.endingContribution -
      lastSettlementBlock_1.startingContribution;
    expect(
      globalSettlement_1.settledAmount - globalSettlement_0.settledAmount
    ).toEqual(rewardAmount);
    expect(
      globalSettlement_1.settlementBlocksLastRewardPoolContribution -
        globalSettlement_0.settlementBlocksLastRewardPoolContribution
    ).toEqual(globalSettledContribution_1);

    // update user1 reward
    await user1.reward.updatePools.execute(null);

    const user1Reward_1 = (await user1.reward.resolve(true))!;

    // check user1 reward pool settlement amount delta
    const user1Settlement_0 = user1Reward_0.basePool.settlements[0];
    const user1Settlement_1 = user1Reward_1.basePool.settlements[0];
    const user1SettledContribution_1 =
      user1Settlement_1.settledContribution -
      user1Settlement_0.settledContribution;
    expect(
      user1Settlement_1.settledAmount - user1Settlement_0.settledAmount
    ).toEqual(
      (rewardAmount * user1SettledContribution_1) / globalSettledContribution_1
    );

    // user1 claims
    const user1ClaimRes = await user1.reward.claim.execute(
      {
        mint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
        amount: null,
        recipient: null,
      },
      { signers: [signer1] }
    );

    const user1Reward_2 = (await user1.reward.resolve(true))!;

    const user1Settlement_2 = user1Reward_2.basePool.settlements[0];
    expect(
      user1ClaimRes.events?.userClaimedReward?.claimedRewardTokenAmount
    ).toEqual(
      user1Settlement_1.settledAmount! - user1Settlement_1.claimedAmount!
    );

    // user2 claim total
    await user2.reward.claim.execute(
      {
        mint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
        amount: null,
        recipient: null,
      },
      { signers: [signer2] }
    );

    const user2Reward_2 = (await user2.reward.resolve(true))!;

    // check user2 reward pool settlement amount delta
    const user2Settlement_0 = user2Reward_0.basePool.settlements[0];
    const user2Settlement_2 = user2Reward_2.basePool.settlements[0];
    const user2SettledContribution_1 =
      user2Settlement_2.settledContribution -
      user2Settlement_0.settledContribution;
    expect(
      user2Settlement_2.settledAmount - user2Settlement_0.settledAmount
    ).toEqual(
      (rewardAmount * user2SettledContribution_1) / globalSettledContribution_1
    );

    // update
    await ctx.reward.updatePools.execute(null);
    await user1.reward.updatePools.execute(null);

    const globalReward_2 = (await ctx.reward.resolve(true))!;

    // check remaing amount == 1
    const globalSettlement_2 = globalReward_2.basePool.settlements[0];
    expect(
      globalSettlement_2.remainingAmount - globalSettlement_1.remainingAmount
    ).toEqual(1n);

    expect(
      globalSettlement_2.claimedAmount - globalSettlement_1.claimedAmount
    ).toEqual(
      user1Settlement_2.claimedAmount -
        user1Settlement_1.claimedAmount +
        user2Settlement_2.claimedAmount -
        user2Settlement_0.claimedAmount
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

  // TODO: full operation test through cash in/out flow
  /** 6. operation cycle */
  test('fragBTC does restake/unrestake assets into/from solvBTC vault', async () => {
    // no changes on fragBTC price
    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "MASKED(__lookupTableAddress)",
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
        "depositResidualMicroReceiptTokenAmount": 999995n,
        "metadata": null,
        "normalizedToken": null,
        "oneReceiptTokenAsSOL": 647695086769n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "receiptTokenSupply": 5620987629n,
        "restakingVaultReceiptTokens": [
          {
            "mint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
          },
          {
            "mint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
          },
          {
            "mint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 8,
            "depositable": true,
            "mint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            "oneTokenAsReceiptToken": 99999999n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 2181481475n,
            "operationTotalAmount": 2181481475n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 2181481474n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 999999n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "oneTokenAsReceiptToken": 99999999n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 1487654321n,
            "operationTotalAmount": 1487654321n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 1487654320n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 16418n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
            "oneTokenAsReceiptToken": 99999999n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 1951851835n,
            "operationTotalAmount": 1951851835n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 1951851834n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);
    await ctx.fund.runCommand.executeChained(null);
    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "MASKED(__lookupTableAddress)",
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
        "depositResidualMicroReceiptTokenAmount": 999995n,
        "metadata": null,
        "normalizedToken": null,
        "oneReceiptTokenAsSOL": 647695086769n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "receiptTokenSupply": 5620987629n,
        "restakingVaultReceiptTokens": [
          {
            "mint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
            "oneReceiptTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 2174937031n,
            "operationTotalAmount": 2174937031n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
          },
          {
            "mint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
            "oneReceiptTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 1483191359n,
            "operationTotalAmount": 1483191359n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
          },
          {
            "mint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
            "oneReceiptTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 1945996280n,
            "operationTotalAmount": 1945996280n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 8,
            "depositable": true,
            "mint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            "oneTokenAsReceiptToken": 99999999n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 6544444n,
            "operationTotalAmount": 6544444n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 2181481474n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 999999n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "oneTokenAsReceiptToken": 99999999n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 4462962n,
            "operationTotalAmount": 4462962n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 1487654320n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 16418n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
            "oneTokenAsReceiptToken": 99999999n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 5855555n,
            "operationTotalAmount": 5855555n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 1951851834n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);

    // mock solv operation and check fragBTC price again
    await validator.airdropToken(
      solv.zBTC.address!,
      'SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8',
      30_0000_0000n
    );
    await solv.zBTC.setSolvProtocolWallet.execute({ address: feePayer });

    // transfer zBTC to solv protocol
    await expect(
      solv.zBTC.confirmDeposits.execute(null)
    ).resolves.not.toThrow();
    await ctx.fund.updatePrices.execute(null);
    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "MASKED(__lookupTableAddress)",
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
        "depositResidualMicroReceiptTokenAmount": 999995n,
        "metadata": null,
        "normalizedToken": null,
        "oneReceiptTokenAsSOL": 647695086769n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "receiptTokenSupply": 5620987629n,
        "restakingVaultReceiptTokens": [
          {
            "mint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
            "oneReceiptTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 2174937031n,
            "operationTotalAmount": 2174937031n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
          },
          {
            "mint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
            "oneReceiptTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 1483191359n,
            "operationTotalAmount": 1483191359n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
          },
          {
            "mint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
            "oneReceiptTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 1945996280n,
            "operationTotalAmount": 1945996280n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 8,
            "depositable": true,
            "mint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            "oneTokenAsReceiptToken": 99999999n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 6544444n,
            "operationTotalAmount": 6544444n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 2181481474n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 999999n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "oneTokenAsReceiptToken": 99999999n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 4462962n,
            "operationTotalAmount": 4462962n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 1487654320n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 16418n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
            "oneTokenAsReceiptToken": 99999999n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 5855555n,
            "operationTotalAmount": 5855555n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 1951851834n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);

    // redeem first solvBTC.jup from solv protocol to the solv vault
    await expect(
      solv.zBTC.completeDeposits.execute({
        redeemedSolvReceiptTokenAmount: 21_7493_7031n,
        newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_0000_0000_000000n,
      })
    ).resolves.not.toThrow();
    await ctx.fund.updatePrices.execute(null);

    // should have no changes on prices yet
    // fragBTC: 647695086769n, zBTC VRT: 647695086539n -> 647695086769n, 647695086539n
    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "MASKED(__lookupTableAddress)",
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
        "depositResidualMicroReceiptTokenAmount": 999995n,
        "metadata": null,
        "normalizedToken": null,
        "oneReceiptTokenAsSOL": 647695086769n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "receiptTokenSupply": 5620987629n,
        "restakingVaultReceiptTokens": [
          {
            "mint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
            "oneReceiptTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 2174937031n,
            "operationTotalAmount": 2174937031n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
          },
          {
            "mint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
            "oneReceiptTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 1483191359n,
            "operationTotalAmount": 1483191359n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
          },
          {
            "mint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
            "oneReceiptTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 1945996280n,
            "operationTotalAmount": 1945996280n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 8,
            "depositable": true,
            "mint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            "oneTokenAsReceiptToken": 99999999n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 6544444n,
            "operationTotalAmount": 6544444n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 2181481474n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 999999n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "oneTokenAsReceiptToken": 99999999n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 4462962n,
            "operationTotalAmount": 4462962n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 1487654320n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 16418n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
            "oneTokenAsReceiptToken": 99999999n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 5855555n,
            "operationTotalAmount": 5855555n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 1951851834n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);

    // now mock increased solvBTC.jup price
    await user1.deposit.execute(
      {
        assetMint: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
        assetAmount: 1_0000_0000n,
      },
      { signers: [signer1] }
    );
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'RestakeVST',
      operator: restaking.knownAddresses.fundManager,
    });
    await expect(
      solv.zBTC.confirmDeposits.execute(null)
    ).resolves.not.toThrow();
    await expect(
      solv.zBTC.completeDeposits.execute({
        redeemedSolvReceiptTokenAmount: (1_0000_0000n * 10n) / 11n,
        newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_1000_0000_000000n,
      })
    ).resolves.not.toThrow();

    // fragBTC: 647695086769n, zBTC VRT: 647695086539n -> 672352352226n (x1.038...), 709711095573n (x1.095)
    await ctx.fund.updatePrices.execute(null);
    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "MASKED(__lookupTableAddress)",
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
        "depositResidualMicroReceiptTokenAmount": 964414n,
        "metadata": null,
        "normalizedToken": null,
        "oneReceiptTokenAsSOL": 672352352226n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "receiptTokenSupply": 5720987629n,
        "restakingVaultReceiptTokens": [
          {
            "mint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
            "oneReceiptTokenAsSol": 709711095573n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 2274637031n,
            "operationTotalAmount": 2274637031n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
          },
          {
            "mint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
            "oneReceiptTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 1483191359n,
            "operationTotalAmount": 1483191359n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
          },
          {
            "mint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
            "oneReceiptTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 1945996280n,
            "operationTotalAmount": 1945996280n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 8,
            "depositable": true,
            "mint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            "oneTokenAsReceiptToken": 96332686n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 6844444n,
            "operationTotalAmount": 6844444n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 2407618931n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 999999n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "oneTokenAsReceiptToken": 96332686n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 4462962n,
            "operationTotalAmount": 4462962n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 1433097379n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 16418n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
            "oneTokenAsReceiptToken": 96332686n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 5855555n,
            "operationTotalAmount": 5855555n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 1880271317n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);

    // now withdrawal test
    await user1.requestWithdrawal.execute(
      {
        assetMint: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
        receiptTokenAmount: 1_0000_0000n,
      },
      { signers: [signer1] }
    );
    await expectMasked(user1.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "lamports": 99962596960n,
        "maxWithdrawalRequests": 4,
        "receiptTokenAmount": 5520987629n,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "supportedAssets": [
          {
            "amount": 97718237044n,
            "decimals": 8,
            "depositable": true,
            "mint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
          },
          {
            "amount": 98512158025n,
            "decimals": 8,
            "depositable": true,
            "mint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
          },
          {
            "amount": 98148148165n,
            "decimals": 8,
            "depositable": true,
            "mint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
          },
        ],
        "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
        "withdrawalRequests": [
          {
            "batchId": 2n,
            "createdAt": "MASKED(/.*At?$/)",
            "receiptTokenAmount": 100000000n,
            "requestId": 3n,
            "state": "cancelable",
            "supportedAssetMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
          },
        ],
        "wrappedTokenAmount": 0n,
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);

    await ctx.fund.runCommand.executeChained(null);
    await expectMasked(user1.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "lamports": 99962596960n,
        "maxWithdrawalRequests": 4,
        "receiptTokenAmount": 5520987629n,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "supportedAssets": [
          {
            "amount": 97718237044n,
            "decimals": 8,
            "depositable": true,
            "mint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
          },
          {
            "amount": 98512158025n,
            "decimals": 8,
            "depositable": true,
            "mint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
          },
          {
            "amount": 98148148165n,
            "decimals": 8,
            "depositable": true,
            "mint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
          },
        ],
        "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
        "withdrawalRequests": [
          {
            "batchId": 2n,
            "createdAt": "MASKED(/.*At?$/)",
            "receiptTokenAmount": 100000000n,
            "requestId": 3n,
            "state": "processing",
            "supportedAssetMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
          },
        ],
        "wrappedTokenAmount": 0n,
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);

    // airdrop enough extra yields and original vst to solv protocol wallet
    await expect(
      solv.zBTC.confirmWithdrawalRequests.execute(null)
    ).resolves.not.toThrow();
    await validator.airdropToken(
      solv.zBTC.address!,
      'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
      10_0000_0000n
    );

    // process withdrawal from solv vault
    await expect(
      solv.zBTC.completeWithdrawalRequests.execute({
        burntSolvReceiptTokenAmount: 88147707n,
        redeemedSupportedTokenAmount: (88147707n * 110n) / 100n,
        oldOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_1000_0000_000000n,
      })
    ).resolves.not.toThrow();
    await expect(solv.zBTC.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
          "rewardManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "solvManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [
          {
            "amount": 0n,
            "delegate": {
              "__option": "Some",
              "value": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
            },
            "mint": "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
          },
        ],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 109574877225154n,
        "oneReceiptTokenAsSupportedTokenAmount": 109574877n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 110000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 110000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 2186147332n,
        "solvProtocolDepositFeeRate": 0,
        "solvProtocolWallet": "GiDkDCZjVC8Nk1Fd457qGSV2g3MQX62n7cV5CvgFyGfF",
        "solvProtocolWithdrawalFeeRate": 0,
        "solvReceiptTokenAmount": 2911852293n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 2177698414n,
        "supportedTokenAmount": 1000000000n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 0n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 88489699n,
            "requests": [
              {
                "id": 1n,
                "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 110000000000000n,
                "oneSolvReceiptTokenAsSupportedTokenAmount": 110000000n,
                "receiptTokenEnqueuedAmount": 88489699n,
                "solvReceiptTokenLockedAmount": 88147707n,
                "supportedTokenLockedAmount": 0n,
                "supportedTokenTotalEstimatedAmount": 96962478n,
              },
            ],
            "supportedTokenDeductedFeeAmount": 1n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 96962477n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 0n,
            "requests": [],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 0n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 0n,
            "requests": [],
            "supportedTokenReceivableAmount": 0n,
          },
        },
      }
    `);
    await ctx.fund.updatePrices.execute(null);

    // note: precision error or safe withdrawal request capacity => fragBTC: 672352352226n, zBTC VRT: 709711095573n -> 672352352135n (x0.9999999999), 709711095335n (x0.9999999997)
    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "MASKED(__lookupTableAddress)",
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
        "depositResidualMicroReceiptTokenAmount": 964414n,
        "metadata": null,
        "normalizedToken": null,
        "oneReceiptTokenAsSOL": 672352352226n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "receiptTokenSupply": 5720987629n,
        "restakingVaultReceiptTokens": [
          {
            "mint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
            "oneReceiptTokenAsSol": 709711095868n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 2186147332n,
            "operationTotalAmount": 2186147332n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 96962478n,
            "vault": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
          },
          {
            "mint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
            "oneReceiptTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 1483191359n,
            "operationTotalAmount": 1483191359n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
          },
          {
            "mint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
            "oneReceiptTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 1945996280n,
            "operationTotalAmount": 1945996280n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 8,
            "depositable": true,
            "mint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            "oneTokenAsReceiptToken": 96332686n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 96962478n,
            "operationReservedAmount": 6844444n,
            "operationTotalAmount": 103806922n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 2307618931n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 999999n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "oneTokenAsReceiptToken": 96332686n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 4462962n,
            "operationTotalAmount": 4462962n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 1433097379n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 16418n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
            "oneTokenAsReceiptToken": 96332686n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 5855555n,
            "operationTotalAmount": 5855555n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 1880271317n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);

    await ctx.fund.runCommand.executeChained(null);

    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "MASKED(__lookupTableAddress)",
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
        "depositResidualMicroReceiptTokenAmount": 964414n,
        "metadata": null,
        "normalizedToken": null,
        "oneReceiptTokenAsSOL": 672352352678n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "receiptTokenSupply": 5620987629n,
        "restakingVaultReceiptTokens": [
          {
            "mint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
            "oneReceiptTokenAsSol": 709711095868n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 2186147332n,
            "operationTotalAmount": 2186147332n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
          },
          {
            "mint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
            "oneReceiptTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 1483191359n,
            "operationTotalAmount": 1483191359n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "5zXiPsDznkiEA4nKvWEWuJEYBupPEBAdA1Qnb7j25PdJ",
          },
          {
            "mint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
            "oneReceiptTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 1945996280n,
            "operationTotalAmount": 1945996280n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unrestakingAmountAsSupportedToken": 0n,
            "vault": "E8GGZBniH85AGo2oGHEf6VeBWEHs3u8SN8iiyUsMV82B",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 8,
            "depositable": true,
            "mint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            "oneTokenAsReceiptToken": 96332686n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 1n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 1n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 2307618934n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 924919n,
            "withdrawalUserReservedAmount": 103599311n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "oneTokenAsReceiptToken": 96332686n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 4462962n,
            "operationTotalAmount": 4462962n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 1433097378n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 16418n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
            "oneTokenAsReceiptToken": 96332686n,
            "oneTokenAsSol": 647695086539n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 5855555n,
            "operationTotalAmount": 5855555n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 1880271316n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);

    await expectMasked(solv.zBTC.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
          "rewardManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "solvManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [
          {
            "amount": 0n,
            "delegate": {
              "__option": "Some",
              "value": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
            },
            "mint": "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
          },
        ],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 109574877225154n,
        "oneReceiptTokenAsSupportedTokenAmount": 109574877n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 110000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 110000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 2186147332n,
        "solvProtocolDepositFeeRate": 0,
        "solvProtocolWallet": "GiDkDCZjVC8Nk1Fd457qGSV2g3MQX62n7cV5CvgFyGfF",
        "solvProtocolWithdrawalFeeRate": 0,
        "solvReceiptTokenAmount": 2911852293n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 2177698414n,
        "supportedTokenAmount": 903037523n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 0n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 0n,
            "requests": [],
            "supportedTokenDeductedFeeAmount": 0n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 0n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 0n,
            "requests": [],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 0n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 0n,
            "requests": [],
            "supportedTokenReceivableAmount": 0n,
          },
        },
      }
    `);

    await expectMasked(user1.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "lamports": 99962596960n,
        "maxWithdrawalRequests": 4,
        "receiptTokenAmount": 5520987629n,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "supportedAssets": [
          {
            "amount": 97718237044n,
            "decimals": 8,
            "depositable": true,
            "mint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
          },
          {
            "amount": 98512158025n,
            "decimals": 8,
            "depositable": true,
            "mint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
          },
          {
            "amount": 98148148165n,
            "decimals": 8,
            "depositable": true,
            "mint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
          },
        ],
        "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
        "withdrawalRequests": [
          {
            "batchId": 2n,
            "createdAt": "MASKED(/.*At?$/)",
            "receiptTokenAmount": 100000000n,
            "requestId": 3n,
            "state": "claimable",
            "supportedAssetMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
          },
        ],
        "wrappedTokenAmount": 0n,
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);

    await expectMasked(ctx.fund.latestWithdrawalBatches.resolve(true)).resolves
      .toMatchInlineSnapshot(`
      [
        {
          "assetFeeAmount": 207613n,
          "assetUserAmount": 103599311n,
          "batchId": 2n,
          "claimedAssetUserAmount": 0n,
          "claimedReceiptTokenAmount": 0n,
          "numClaimedRequests": 0n,
          "numRequests": 1n,
          "processedAt": "MASKED(/.*At?$/)",
          "receiptTokenAmount": 100000000n,
          "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
          "supportedTokenMint": {
            "__option": "Some",
            "value": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
          },
          "supportedTokenProgram": {
            "__option": "Some",
            "value": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          },
        },
        null,
        null,
      ]
    `);

    await expectMasked(
      user1.withdraw.execute(
        {
          assetMint: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
          requestId: 3n,
        },
        { signers: [signer1] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
          "requestId": 3n,
          "skipUserRewardAccountCreation": false,
        },
        "events": {
          "unknown": [],
          "userWithdrewFromFund": {
            "batchId": 2n,
            "burntReceiptTokenAmount": 100000000n,
            "deductedFeeAmount": 207613n,
            "fundAccount": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
            "fundWithdrawalBatchAccount": "S3G2m8RLtyMRFjqEMkiwidCwJ4PQhgesDhpyFFfv37P",
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "requestId": 3n,
            "returnedReceiptTokenAmount": 0n,
            "supportedTokenMint": {
              "__option": "Some",
              "value": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            },
            "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
            "userFundAccount": "DdVfA4rT4tSJRMi688zb5SeZitH91AcKXU79Q4A3pCHg",
            "userReceiptTokenAccount": "31GDTcPyFTexmiwSvLTZAcY2JSxafoy3yVavK4iAYaLE",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "CHBjDuyN1JxAfeYY7wz68b78E6uPpek1TKqjKeyDJZD6",
            },
            "withdrawnAmount": 103599311n,
          },
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);
  });

  test('fragBTC APY can be estimated through vault supported token compounded amount', async () => {
    /*
     * VRT price can be same OR increased based on how solv protocol operates deposited asset. (We assume VRT price doesn't decrease right now)
     * Test two possible cases whether harvest_restaking_yield command emits correct event.
     * Since harvest command iterate all vaults in fragBTC fund and runCommand returns last result, we use last vault(wBTC vault) for testing.
     */

    // reset VST compounded amount
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestRestakingYield',
      operator: restaking.knownAddresses.fundManager,
    });

    // 1) VRT Price remains same
    // 1-1) VRT Amount remains same
    await expectMasked(ctx.fund.restakingVaults.children[2].resolve(true))
      .resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
          "rewardManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "solvManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100000000n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 1945996280n,
        "solvProtocolDepositFeeRate": 0,
        "solvProtocolWallet": "11111111111111111111111111111111",
        "solvProtocolWithdrawalFeeRate": 0,
        "solvReceiptTokenAmount": 0n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 1945996280n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 1945996280n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 0n,
            "requests": [],
            "supportedTokenDeductedFeeAmount": 0n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 0n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 0n,
            "requests": [],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 0n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 0n,
            "requests": [],
            "supportedTokenReceivableAmount": 0n,
          },
        },
      }
    `);

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'ClaimUnrestakedVST',
      operator: restaking.knownAddresses.fundManager,
    });

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'UnrestakeVRT',
      operator: restaking.knownAddresses.fundManager,
    });

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'RestakeVST',
      operator: restaking.knownAddresses.fundManager,
    });

    // harvest yield
    let res = await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestRestakingYield',
      operator: restaking.knownAddresses.fundManager,
    });

    let evt = res.events!.operatorRanFundCommand!;
    let result = isSome(evt.result)
      ? (evt.result.value
          .fields[0] as restakingTypes.HarvestRestakingYieldCommandResult)
      : null;

    // result value is None since there is no VRT price change, compounded VST amount is 0
    expect(result).toBeNull();

    // 1-2) VRT Amount increases (user deposits vst -> restake vst command executed)
    await validator.airdropToken(
      user1.address!,
      '3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh',
      100_0000_0000n
    );

    await user1.deposit.execute(
      {
        assetMint: '3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh',
        assetAmount: 20_0000_0000n,
      },
      { signers: [signer1] }
    );

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'RestakeVST',
      operator: restaking.knownAddresses.fundManager,
    });

    await expectMasked(ctx.fund.restakingVaults.children[2].resolve(true))
      .resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
          "rewardManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "solvManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100000000n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 3939996280n,
        "solvProtocolDepositFeeRate": 0,
        "solvProtocolWallet": "11111111111111111111111111111111",
        "solvProtocolWithdrawalFeeRate": 0,
        "solvReceiptTokenAmount": 0n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 3939996280n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 3939996280n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 0n,
            "requests": [],
            "supportedTokenDeductedFeeAmount": 0n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 0n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 0n,
            "requests": [],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 0n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 0n,
            "requests": [],
            "supportedTokenReceivableAmount": 0n,
          },
        },
      }
    `);

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'ClaimUnrestakedVST',
      operator: restaking.knownAddresses.fundManager,
    });

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'UnrestakeVRT',
      operator: restaking.knownAddresses.fundManager,
    });

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'RestakeVST',
      operator: restaking.knownAddresses.fundManager,
    });

    // harvest yield
    res = await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestRestakingYield',
      operator: restaking.knownAddresses.fundManager,
    });

    evt = res.events!.operatorRanFundCommand!;
    result = isSome(evt.result)
      ? (evt.result.value
          .fields[0] as restakingTypes.HarvestRestakingYieldCommandResult)
      : null;

    // result value is None since there is no VRT price change, compounded VST amount is 0
    expect(result).toBeNull();

    // 1-3) VRT Amount decreases (user requests withdraw -> full command cycle executed)
    await user1.requestWithdrawal.execute(
      {
        assetMint: '3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh',
        receiptTokenAmount: 10_0000_0000n,
      },
      { signers: [signer1] }
    );

    await ctx.fund.runCommand.executeChained(null);

    await expectMasked(ctx.fund.restakingVaults.children[2].resolve(true))
      .resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
          "rewardManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "solvManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100000000n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 2913782586n,
        "solvProtocolDepositFeeRate": 0,
        "solvProtocolWallet": "11111111111111111111111111111111",
        "solvProtocolWithdrawalFeeRate": 0,
        "solvReceiptTokenAmount": 0n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 3939996280n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 2913782586n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 0n,
            "requests": [],
            "supportedTokenDeductedFeeAmount": 0n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 0n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 1026213694n,
            "requests": [
              {
                "id": 1n,
                "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
                "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
                "receiptTokenEnqueuedAmount": 1026213694n,
                "solvReceiptTokenLockedAmount": 0n,
                "supportedTokenLockedAmount": 1026213694n,
                "supportedTokenTotalEstimatedAmount": 1026213694n,
              },
            ],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 1026213694n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 0n,
            "requests": [],
            "supportedTokenReceivableAmount": 0n,
          },
        },
      }
    `);

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'ClaimUnrestakedVST',
      operator: restaking.knownAddresses.fundManager,
    });

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'UnrestakeVRT',
      operator: restaking.knownAddresses.fundManager,
    });

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'RestakeVST',
      operator: restaking.knownAddresses.fundManager,
    });

    // harvest yield
    res = await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestRestakingYield',
      operator: restaking.knownAddresses.fundManager,
    });

    evt = res.events!.operatorRanFundCommand!;
    result = isSome(evt.result)
      ? (evt.result.value
          .fields[0] as restakingTypes.HarvestRestakingYieldCommandResult)
      : null;

    // result value is None since there is no VRT price change, compounded VST amount is 0
    expect(result).toBeNull();

    // 2) VRT Price increases
    // mock solv operation
    await validator.airdropToken(
      solv.wBTC.address!,
      'SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8',
      100_0000_0000n
    );
    await solv.wBTC.setSolvProtocolWallet.execute({ address: feePayer });

    await expectMasked(ctx.fund.restakingVaults.children[2].resolve(true))
      .resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
          "rewardManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "solvManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100000000n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 2913782586n,
        "solvProtocolDepositFeeRate": 0,
        "solvProtocolWallet": "GiDkDCZjVC8Nk1Fd457qGSV2g3MQX62n7cV5CvgFyGfF",
        "solvProtocolWithdrawalFeeRate": 0,
        "solvReceiptTokenAmount": 10000000000n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 3939996280n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 2913782586n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 0n,
            "requests": [],
            "supportedTokenDeductedFeeAmount": 0n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 0n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 1026213694n,
            "requests": [
              {
                "id": 1n,
                "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
                "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
                "receiptTokenEnqueuedAmount": 1026213694n,
                "solvReceiptTokenLockedAmount": 0n,
                "supportedTokenLockedAmount": 1026213694n,
                "supportedTokenTotalEstimatedAmount": 1026213694n,
              },
            ],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 1026213694n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 0n,
            "requests": [],
            "supportedTokenReceivableAmount": 0n,
          },
        },
      }
    `);

    // user deposits wBTC to trigger VRT price change
    await user1.deposit.execute(
      {
        assetMint: '3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh',
        assetAmount: 10_0000_0000n,
      },
      { signers: [signer1] }
    );

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'RestakeVST',
      operator: restaking.knownAddresses.fundManager,
    });

    await expectMasked(ctx.fund.restakingVaults.children[2].resolve(true))
      .resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
          "rewardManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "solvManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100000000n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 2913782586n,
        "solvProtocolDepositFeeRate": 0,
        "solvProtocolWallet": "GiDkDCZjVC8Nk1Fd457qGSV2g3MQX62n7cV5CvgFyGfF",
        "solvProtocolWithdrawalFeeRate": 0,
        "solvReceiptTokenAmount": 10000000000n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 3939996280n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 2913782586n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 0n,
            "requests": [],
            "supportedTokenDeductedFeeAmount": 0n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 0n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 1026213694n,
            "requests": [
              {
                "id": 1n,
                "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
                "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
                "receiptTokenEnqueuedAmount": 1026213694n,
                "solvReceiptTokenLockedAmount": 0n,
                "supportedTokenLockedAmount": 1026213694n,
                "supportedTokenTotalEstimatedAmount": 1026213694n,
              },
            ],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 1026213694n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 0n,
            "requests": [],
            "supportedTokenReceivableAmount": 0n,
          },
        },
      }
    `);

    await solv.wBTC.confirmDeposits.execute(null);

    await expectMasked(ctx.fund.restakingVaults.children[2].resolve(true))
      .resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
          "rewardManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "solvManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100000000n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 2913782586n,
        "solvProtocolDepositFeeRate": 0,
        "solvProtocolWallet": "GiDkDCZjVC8Nk1Fd457qGSV2g3MQX62n7cV5CvgFyGfF",
        "solvProtocolWithdrawalFeeRate": 0,
        "solvReceiptTokenAmount": 10000000000n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 2913782586n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 1026213694n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 0n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 0n,
            "requests": [],
            "supportedTokenDeductedFeeAmount": 0n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 0n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 1026213694n,
            "requests": [
              {
                "id": 1n,
                "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
                "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
                "receiptTokenEnqueuedAmount": 1026213694n,
                "solvReceiptTokenLockedAmount": 0n,
                "supportedTokenLockedAmount": 1026213694n,
                "supportedTokenTotalEstimatedAmount": 1026213694n,
              },
            ],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 1026213694n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 0n,
            "requests": [],
            "supportedTokenReceivableAmount": 0n,
          },
        },
      }
    `);

    // trigger VRT price change by increasing SRT value ***

    const priceIncreasedAmountAsPercent = 10n;
    const receivableSolvReceiptTokenAmount = 29_1378_2586n;
    const expectedSupportedTokenAmount = receivableSolvReceiptTokenAmount;

    await solv.wBTC.completeDeposits.execute({
      redeemedSolvReceiptTokenAmount: receivableSolvReceiptTokenAmount,
      newOneSolvReceiptTokenAsMicroSupportedTokenAmount:
        (1_0000_0000_000000n * (100n + priceIncreasedAmountAsPercent)) / 100n,
    });

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'ClaimUnrestakedVST',
      operator: restaking.knownAddresses.fundManager,
    });

    // harvest yield - VST compounded amount should be 10%
    res = await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'HarvestRestakingYield',
      operator: restaking.knownAddresses.fundManager,
    });

    evt = res.events!.operatorRanFundCommand!;
    result = isSome(evt.result)
      ? (evt.result.value
          .fields[0] as restakingTypes.HarvestRestakingYieldCommandResult)
      : null;

    // compounded vst amount should be 10% of expected supported token amount
    expect(result?.vaultSupportedTokenCompoundedAmount).toEqual(
      (expectedSupportedTokenAmount * priceIncreasedAmountAsPercent) / 100n
    );

    // check whether supported_token_compounded_amount value in RestakingVault struct is reset to '0'
    expect(
      await ctx.fund
        .resolveAccount(true)
        .then(
          (fundAccount) =>
            fundAccount!.data.restakingVaults[2].supportedTokenCompoundedAmount
        )
    ).toEqual(0n);
  });

  test('duplicate withdrawal requests can be prevented by adopting pending_supported_token_unrestaking_amount', async () => {
    await ctx.fund.runCommand.executeChained(null);

    await validator.airdropToken(
      solv.cbBTC.address!,
      'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
      10_0000_0000n
    );

    await validator.airdropToken(
      solv.cbBTC.address!,
      'SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8',
      10_0000_0000n
    );

    await expectMasked(ctx.fund.restakingVaults.children[1].resolve(true))
      .resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
          "rewardManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "solvManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100000000n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 1483191359n,
        "solvProtocolDepositFeeRate": 0,
        "solvProtocolWallet": "11111111111111111111111111111111",
        "solvProtocolWithdrawalFeeRate": 0,
        "solvReceiptTokenAmount": 1000000000n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 2483191359n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 1483191359n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 0n,
            "requests": [],
            "supportedTokenDeductedFeeAmount": 0n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 0n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 0n,
            "requests": [],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 0n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 0n,
            "requests": [],
            "supportedTokenReceivableAmount": 0n,
          },
        },
      }
    `);

    await validator.airdropToken(
      user1.address!,
      'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
      100_0000_0000n
    );

    // 1) user tries to deposit -> request withdraw. Since there is sufficient amount of tokens in fund reserve,
    // withdraw request shouldn't be enqueued to solv wrapper.
    await user1.deposit.execute(
      {
        assetMint: 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
        assetAmount: 5_0000_0000n,
      },
      { signers: [signer1] }
    );

    await user1.requestWithdrawal.execute(
      {
        assetMint: 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
        receiptTokenAmount: 2_0000_0000n,
      },
      { signers: [signer1] }
    );

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'EnqueueWithdrawalBatch',
      operator: restaking.knownAddresses.fundManager,
    });

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'UnrestakeVRT',
      operator: restaking.knownAddresses.fundManager,
    });

    await expectMasked(ctx.fund.restakingVaults.children[1].resolve(true))
      .resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
          "rewardManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "solvManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100000000n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 1483191359n,
        "solvProtocolDepositFeeRate": 0,
        "solvProtocolWallet": "11111111111111111111111111111111",
        "solvProtocolWithdrawalFeeRate": 0,
        "solvReceiptTokenAmount": 1000000000n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 2483191359n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 1483191359n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 0n,
            "requests": [],
            "supportedTokenDeductedFeeAmount": 0n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 0n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 0n,
            "requests": [],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 0n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 0n,
            "requests": [],
            "supportedTokenReceivableAmount": 0n,
          },
        },
      }
    `);

    // 2) RestakeVST command is executed to mint vrt. When user tries to withdraw repeatedly, only one withdraw request should be enqueued.
    let res = await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'RestakeVST',
      operator: restaking.knownAddresses.fundManager,
    });

    let restakeVstEvt = res.events!.operatorRanFundCommand!;
    let restakeVstResult = isSome(restakeVstEvt.result)
      ? (restakeVstEvt.result.value
          .fields[0] as restakingTypes.RestakeVSTCommandResult)
      : null;

    expect(restakeVstResult!.depositedSupportedTokenAmount).toBeLessThanOrEqual(
      3_0000_0000n
    );

    await expectMasked(ctx.fund.restakingVaults.children[1].resolve(true))
      .resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
          "rewardManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "solvManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100000000n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 1767230386n,
        "solvProtocolDepositFeeRate": 0,
        "solvProtocolWallet": "11111111111111111111111111111111",
        "solvProtocolWithdrawalFeeRate": 0,
        "solvReceiptTokenAmount": 1000000000n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 2767230386n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 1767230386n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 0n,
            "requests": [],
            "supportedTokenDeductedFeeAmount": 0n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 0n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 0n,
            "requests": [],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 0n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 0n,
            "requests": [],
            "supportedTokenReceivableAmount": 0n,
          },
        },
      }
    `);

    await user1.requestWithdrawal.execute(
      {
        assetMint: 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
        receiptTokenAmount: 1_0000_0000n,
      },
      { signers: [signer1] }
    );

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'EnqueueWithdrawalBatch',
      operator: restaking.knownAddresses.fundManager,
    });

    // 1st Unrestake command
    res = await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'UnrestakeVRT',
      operator: restaking.knownAddresses.fundManager,
    });

    let unrestakeVrtEvt = res.events!.operatorRanFundCommand!;
    let unrestakeVrtResult = isSome(unrestakeVrtEvt.result)
      ? (unrestakeVrtEvt.result.value
          .fields[0] as restakingTypes.UnrestakeVRTCommandResult)
      : null;

    expect(unrestakeVrtResult!.unrestakingTokenAmount).toBeGreaterThanOrEqual(
      1_0000_0000n
    );

    await expectMasked(ctx.fund.restakingVaults.children[1].resolve(true))
      .resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
          "rewardManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "solvManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100000000n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 1665962861n,
        "solvProtocolDepositFeeRate": 0,
        "solvProtocolWallet": "11111111111111111111111111111111",
        "solvProtocolWithdrawalFeeRate": 0,
        "solvReceiptTokenAmount": 1000000000n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 2767230386n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 1665962861n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 0n,
            "requests": [],
            "supportedTokenDeductedFeeAmount": 0n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 0n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 101267525n,
            "requests": [
              {
                "id": 1n,
                "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
                "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
                "receiptTokenEnqueuedAmount": 101267525n,
                "solvReceiptTokenLockedAmount": 0n,
                "supportedTokenLockedAmount": 101267525n,
                "supportedTokenTotalEstimatedAmount": 101267525n,
              },
            ],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 101267525n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 0n,
            "requests": [],
            "supportedTokenReceivableAmount": 0n,
          },
        },
      }
    `);

    // check pending amount value
    expect(
      await ctx.fund
        .resolveAccount(true)
        .then(
          (fundAccount) =>
            fundAccount!.data.restakingVaults[1]
              .pendingSupportedTokenUnrestakingAmount
        )
    ).toEqual(101267525n);

    // 2nd Unrestake command
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'UnrestakeVRT',
      operator: restaking.knownAddresses.fundManager,
    });

    await expectMasked(ctx.fund.restakingVaults.children[1].resolve(true))
      .resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
          "rewardManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "solvManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100000000n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 1665962861n,
        "solvProtocolDepositFeeRate": 0,
        "solvProtocolWallet": "11111111111111111111111111111111",
        "solvProtocolWithdrawalFeeRate": 0,
        "solvReceiptTokenAmount": 1000000000n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 2767230386n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 1665962861n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 0n,
            "requests": [],
            "supportedTokenDeductedFeeAmount": 0n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 0n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 101267525n,
            "requests": [
              {
                "id": 1n,
                "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
                "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
                "receiptTokenEnqueuedAmount": 101267525n,
                "solvReceiptTokenLockedAmount": 0n,
                "supportedTokenLockedAmount": 101267525n,
                "supportedTokenTotalEstimatedAmount": 101267525n,
              },
            ],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 101267525n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 0n,
            "requests": [],
            "supportedTokenReceivableAmount": 0n,
          },
        },
      }
    `);

    // 3rd Unrestake command
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'UnrestakeVRT',
      operator: restaking.knownAddresses.fundManager,
    });

    await expectMasked(ctx.fund.restakingVaults.children[1].resolve(true))
      .resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
          "rewardManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "solvManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100000000n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 1665962861n,
        "solvProtocolDepositFeeRate": 0,
        "solvProtocolWallet": "11111111111111111111111111111111",
        "solvProtocolWithdrawalFeeRate": 0,
        "solvReceiptTokenAmount": 1000000000n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 2767230386n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 1665962861n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 0n,
            "requests": [],
            "supportedTokenDeductedFeeAmount": 0n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 0n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 101267525n,
            "requests": [
              {
                "id": 1n,
                "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
                "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
                "receiptTokenEnqueuedAmount": 101267525n,
                "solvReceiptTokenLockedAmount": 0n,
                "supportedTokenLockedAmount": 101267525n,
                "supportedTokenTotalEstimatedAmount": 101267525n,
              },
            ],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 101267525n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 0n,
            "requests": [],
            "supportedTokenReceivableAmount": 0n,
          },
        },
      }
    `);

    // 3) If user requests more token to withdraw so that pending amount can not afford user requests, additional request should be enqueued.
    await user1.requestWithdrawal.execute(
      {
        assetMint: 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
        receiptTokenAmount: 1_0000_0000n,
      },
      { signers: [signer1] }
    );

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'EnqueueWithdrawalBatch',
      operator: restaking.knownAddresses.fundManager,
    });

    res = await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'UnrestakeVRT',
      operator: restaking.knownAddresses.fundManager,
    });

    unrestakeVrtEvt = res.events!.operatorRanFundCommand!;
    unrestakeVrtResult = isSome(unrestakeVrtEvt.result)
      ? (unrestakeVrtEvt.result.value
          .fields[0] as restakingTypes.UnrestakeVRTCommandResult)
      : null;

    await expectMasked(ctx.fund.restakingVaults.children[1].resolve(true))
      .resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
          "rewardManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "solvManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100000000n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 1558732374n,
        "solvProtocolDepositFeeRate": 0,
        "solvProtocolWallet": "11111111111111111111111111111111",
        "solvProtocolWithdrawalFeeRate": 0,
        "solvReceiptTokenAmount": 1000000000n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 2767230386n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 1558732374n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 0n,
            "requests": [],
            "supportedTokenDeductedFeeAmount": 0n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 0n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 208498012n,
            "requests": [
              {
                "id": 1n,
                "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
                "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
                "receiptTokenEnqueuedAmount": 101267525n,
                "solvReceiptTokenLockedAmount": 0n,
                "supportedTokenLockedAmount": 101267525n,
                "supportedTokenTotalEstimatedAmount": 101267525n,
              },
              {
                "id": 2n,
                "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
                "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
                "receiptTokenEnqueuedAmount": 107230487n,
                "solvReceiptTokenLockedAmount": 0n,
                "supportedTokenLockedAmount": 107230487n,
                "supportedTokenTotalEstimatedAmount": 107230487n,
              },
            ],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 208498012n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 0n,
            "requests": [],
            "supportedTokenReceivableAmount": 0n,
          },
        },
      }
    `);

    // check pending amount value
    expect(
      await ctx.fund
        .resolveAccount(true)
        .then(
          (fundAccount) =>
            fundAccount!.data.restakingVaults[1]
              .pendingSupportedTokenUnrestakingAmount
        )
    ).toEqual(208498012n);

    // 4) run claim command to flush enqueued request and set pending_supported_token_amount field to '0'
    await solv.cbBTC.setSolvProtocolWallet.execute({ address: feePayer });
    await solv.cbBTC.confirmWithdrawalRequests.execute(null);

    await expectMasked(ctx.fund.restakingVaults.children[1].resolve(true))
      .resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
          "rewardManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "solvManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100000000n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 1558732374n,
        "solvProtocolDepositFeeRate": 0,
        "solvProtocolWallet": "GiDkDCZjVC8Nk1Fd457qGSV2g3MQX62n7cV5CvgFyGfF",
        "solvProtocolWithdrawalFeeRate": 0,
        "solvReceiptTokenAmount": 1000000000n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 2767230386n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 1558732374n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 0n,
            "requests": [],
            "supportedTokenDeductedFeeAmount": 0n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 0n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 0n,
            "requests": [],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 208498012n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 208498012n,
            "requests": [
              {
                "id": 1n,
                "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
                "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
                "receiptTokenEnqueuedAmount": 101267525n,
                "solvReceiptTokenLockedAmount": 0n,
                "supportedTokenLockedAmount": 101267525n,
                "supportedTokenTotalEstimatedAmount": 101267525n,
              },
              {
                "id": 2n,
                "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
                "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
                "receiptTokenEnqueuedAmount": 107230487n,
                "solvReceiptTokenLockedAmount": 0n,
                "supportedTokenLockedAmount": 107230487n,
                "supportedTokenTotalEstimatedAmount": 107230487n,
              },
            ],
            "supportedTokenReceivableAmount": 208498012n,
          },
        },
      }
    `);

    await solv.cbBTC.completeWithdrawalRequests.execute({
      burntSolvReceiptTokenAmount: 0n,
      redeemedSupportedTokenAmount: 0n,
      oldOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_0000_0000_000000n,
    });

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'ClaimUnrestakedVST',
      operator: restaking.knownAddresses.fundManager,
    });

    await expectMasked(ctx.fund.restakingVaults.children[1].resolve(true))
      .resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
          "rewardManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "solvManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100000000n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 1558732374n,
        "solvProtocolDepositFeeRate": 0,
        "solvProtocolWallet": "GiDkDCZjVC8Nk1Fd457qGSV2g3MQX62n7cV5CvgFyGfF",
        "solvProtocolWithdrawalFeeRate": 0,
        "solvReceiptTokenAmount": 1000000000n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 2558732374n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 1558732374n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 0n,
            "requests": [],
            "supportedTokenDeductedFeeAmount": 0n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 0n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 0n,
            "requests": [],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 0n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 0n,
            "requests": [],
            "supportedTokenReceivableAmount": 0n,
          },
        },
      }
    `);

    // check pending amount value
    expect(
      await ctx.fund
        .resolveAccount(true)
        .then(
          (fundAccount) =>
            fundAccount!.data.restakingVaults[1]
              .pendingSupportedTokenUnrestakingAmount
        )
    ).toEqual(0n);
  });

  test('user can deposit srt and receive rt', async () => {
    const assetMint = solv.zBTC.receiptTokenMint.address!;

    // 1. deposit srt to vault
    const user1Solv = solv.zBTC.user(signer1);

    // compare expected VRT amount with actually minted amount with random amount of SRT
    for (let i = 1n; i <= 10n; i++) {
      const previousUser1ReceiptTokenAmount = user1Solv.vaultReceiptTokenAccount
        .account
        ? user1Solv.vaultReceiptTokenAccount.account.data.amount
        : 0n;

      const amountToDeposit = 12_3456_7890n * i;

      await solv.zBTC.resolve(true);
      const vaultData = solv.zBTC.account!.data;

      const vrtSupply = vaultData.vrtSupply;

      const netAssetValueBefore =
        vaultData.vstOperationReservedAmount +
        vaultData.vstOperationReceivableAmount +
        (vaultData.srtOperationReservedAmount * vaultData.oneSrtAsMicroVst) /
          (1_0000_0000n * 1_000_000n) +
        (vaultData.srtOperationReceivableAmount * vaultData.oneSrtAsMicroVst) /
          (1_0000_0000n * 1_000_000n);

      const netAssetValueAfter =
        vaultData.vstOperationReservedAmount +
        vaultData.vstOperationReceivableAmount +
        ((vaultData.srtOperationReservedAmount + amountToDeposit) *
          vaultData.oneSrtAsMicroVst) /
          (1_0000_0000n * 1_000_000n) +
        (vaultData.srtOperationReceivableAmount * vaultData.oneSrtAsMicroVst) /
          (1_0000_0000n * 1_000_000n);

      const expectedVRTAmount =
        ((netAssetValueAfter - netAssetValueBefore) * vrtSupply) /
        netAssetValueBefore;

      const resSolv = await user1Solv.deposit.execute(
        {
          srtAmount: amountToDeposit,
        },
        { signers: [signer1] }
      );

      await user1Solv.resolve(true);
      const currentUser1ReceiptTokenAmount = user1Solv.vaultReceiptTokenAccount
        .account
        ? user1Solv.vaultReceiptTokenAccount.account.data.amount
        : 0n;

      let tokenAmountDiff =
        currentUser1ReceiptTokenAmount - previousUser1ReceiptTokenAmount;

      expect(tokenAmountDiff).toEqual(expectedVRTAmount);
    }

    // 2. deposit vrt to fund
    const fund_1 = await ctx.fund.resolveAccount(true);
    const user1_1 = await user1.resolve(true);

    // deposit fails if tries to deposit specific amount of vault receipt token
    await expect(
      user1.deposit.execute(
        { assetMint, assetAmount: 10_000_000_000n }, // 100 vrt
        { signers: [signer1] }
      )
    ).rejects.toThrowError();

    // deposit fails if tries to deposit non supported token nor non vault receipt token
    await expect(
      user1.deposit.execute(
        { assetMint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq' },
        { signers: [signer1] }
      )
    ).rejects.toThrowError();

    const res_1 = await user1.deposit.execute(
      {
        assetMint,
      },
      { signers: [signer1] }
    );
    await expectMasked(res_1).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": null,
          "assetMint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
          "metadata": null,
          "skipUserFundAccountCreation": false,
          "skipUserRewardAccountCreation": false,
        },
        "events": {
          "unknown": [],
          "userDepositedToVault": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 68164673500n,
            "fundAccount": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
            "mintedReceiptTokenAmount": 69654964332n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "updatedUserRewardAccounts": [
              "8XYL5QhcGSVZFX1wCdvVatXhehd7fwx9CYY4og1Fobt9",
            ],
            "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
            "userFundAccount": "DdVfA4rT4tSJRMi688zb5SeZitH91AcKXU79Q4A3pCHg",
            "userReceiptTokenAccount": "31GDTcPyFTexmiwSvLTZAcY2JSxafoy3yVavK4iAYaLE",
            "userVaultReceiptTokenAccount": "8ZDbg1SBU2BdpQ5kD8LAzGswWZSXPgvWzRjQrmmwqu6n",
            "vaultAccount": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
            "vaultReceiptTokenMint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
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

    const fund_2 = await ctx.fund.resolveAccount(true);
    const user1_2 = await user1.resolve(true);
    expect(
      fund_2?.data.restakingVaults[0].receiptTokenOperationReservedAmount! -
        fund_1?.data.restakingVaults[0].receiptTokenOperationReservedAmount!
    ).toEqual(user1Solv.vaultReceiptTokenAccount.account?.data.amount);
    expect(user1_2?.receiptTokenAmount! - user1_1?.receiptTokenAmount!).toEqual(
      res_1.events?.userDepositedToVault?.mintedReceiptTokenAmount
    );
  });

  test('new supported token should be pegged to registered token if pricing source of the registered one is manipulatable', async () => {
    // when there is supported token that uses 'OrcaDEXLiquidityPool' as pricing source,
    // pricing source of new supported token should be pegged to the token that uses 'OrcaDEXLiquidityPool'
    await expect(
      ctx.fund.addSupportedToken.execute({
        mint: '6DNSN2BJsaPFdFFc1zP37kkeNe4Usc1Sqkzr9C9vPWcU',
        pricingSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: '4PTzufF599o1HMtP9HCWkeo6Wk1NoVrRQp9sgf7X6rmB',
        },
      })
    ).rejects.toThrowError();

    await ctx.fund.addSupportedToken.execute({
      mint: '6DNSN2BJsaPFdFFc1zP37kkeNe4Usc1Sqkzr9C9vPWcU',
      pricingSource: {
        __kind: 'PeggedToken',
        address: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
      },
    });

    // sol can't be depositable if there is supported token that uses 'OrcaDEXLiquidityPool' as its pricing source
    await expect(
      ctx.fund.updateAssetStrategy.execute({
        tokenMint: null,
        solDepositable: true,
      })
    ).rejects.toThrowError();

    // restore previous state
    await ctx.fund.removeSupportedToken.execute({
      mint: '6DNSN2BJsaPFdFFc1zP37kkeNe4Usc1Sqkzr9C9vPWcU',
    });
  });
});
