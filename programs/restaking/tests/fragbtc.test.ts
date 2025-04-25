import type { restakingTypes } from '@fragmetric-labs/sdk';
import { isSome } from '@solana/kit';
import { afterAll, beforeAll, describe, expect, test } from 'vitest';
import { createTestSuiteContext, expectMasked } from '../../testutil';
import { initializeFragBTC } from './fragbtc.init';

describe('restaking.fragBTC test', async () => {
  const testCtx = initializeFragBTC(
    await createTestSuiteContext({ programs: { solv: true } })
  );

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  const { validator, feePayer, restaking, initializationTasks } = testCtx;
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

  test(`restaking.fragBTC.reward.resolve`, async () => {
    await expectMasked(ctx.reward.resolve(true)).resolves
      .toMatchInlineSnapshot(`
      {
        "basePool": {
          "contribution": "MASKED(contribution)",
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
          "contribution": "MASKED(contribution)",
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
          {
            "claimable": true,
            "decimals": 6,
            "description": "ZEUS Incentive",
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
            "userFundAccount": "DdVfA4rT4tSJRMi688zb5SeZitH91AcKXU79Q4A3pCHg",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
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
        "metadata": null,
        "oneReceiptTokenAsSOL": 647695086539n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "receiptTokenSupply": 100000000n,
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
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
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
            "userFundAccount": "4qyutWBhvuM6k73kehvo6drqQg93aLJaEiPo4ticeQE1",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
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
        "metadata": null,
        "oneReceiptTokenAsSOL": 647695086539n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "receiptTokenSupply": 300000000n,
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
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);
  });

  /** 3. opeartion and reward **/
  test('fund can settle distributing rewards by operation', async () => {
    // drop distribution token to the vault
    const zBTCVault =
      (await ctx.fund.restakingVaults.children[0]!.resolveAddress())!;
    await validator.airdropToken(
      zBTCVault,
      'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
      100_000_000_000n
    );
    // TODO: delegate ATA (signer: vault)

    // run operator to harvest
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
            "numOperated": 22n,
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
            "tokenAccumulatedDepositAmount": 100000000n,
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
            "tokenAccumulatedDepositAmount": 100000000n,
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
            "tokenAccumulatedDepositAmount": 100000000n,
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

    // TODO: check user reward accounts
    await user1.reward.updatePools.execute(null);
    await expectMasked(user1.reward.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "basePool": {
          "contribution": "MASKED(contribution)",
          "settlements": [],
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
          "contribution": "MASKED(contribution)",
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
              "settledContribution": 0n,
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
        "delegate": "11111111111111111111111111111111",
        "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
        "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
      }
    `);
  });

  /** 4. withdraw and pegging **/
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

    await expect(
      ctx
        .resolve(true)
        .then((data) =>
          data!.supportedAssets.reduce(
            (sum, asset) => sum + asset.operationTotalAmount,
            0n
          )
        ),
      'sum of all underlying assets must be equal to receipt token supply (fund accounting)'
    ).resolves.toEqual(expectedReceiptTokenSupply);

    await expect(
      ctx.fund.reserve.supportedTokens
        .resolve(true)
        .then((accounts) =>
          accounts.reduce(
            (sum, tokenAccount) =>
              tokenAccount ? sum + tokenAccount.amount : sum,
            0n
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

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      receiptTokenSupply: expectedReceiptTokenSupply,
    });
  });
});
