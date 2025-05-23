import { createKeyPairSignerFromBytes } from '@solana/kit';
import { afterAll, beforeAll, describe, expect, test } from 'vitest';
import { createTestSuiteContext, expectMasked } from '../../testutil';
import { initializeFragJTO } from './fragjto.init';

describe('restaking.fragJTO test', async () => {
  const testCtx = initializeFragJTO(await createTestSuiteContext());

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  const { validator, feePayer, restaking, initializationTasks } = testCtx;
  const ctx = restaking.fragJTO;
  const AMOUNT_PER_FRAGJTO = 1_000_000_000n;
  const AMOUNT_PER_WFRAGJTO = 1_000_000_000n;
  const AMOUNT_PER_JTO = 1_000_000_000n;
  const BASIC_ACCRUAL_RATE = 100n;

  const PRICING_DIFF_ERROR_MODIFIER = 100000;

  /* create admin signer (for deposit metadata) */
  const adminSigner = await createKeyPairSignerFromBytes(
    Buffer.from(
      require('../../../keypairs/restaking/shared_local_admin_9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL.json')
    )
  );

  /* create users */
  const [signer1, signer2, signer3, signer4] = await Promise.all([
    validator
      .newSigner('fragJTOTestSigner1', 100n * 1_000_000_000n)
      .then(async (signer) => {
        await Promise.all([
          validator.airdropToken(
            signer.address,
            'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
            1_000n * AMOUNT_PER_JTO
          ),
        ]);
        return signer;
      }),
    validator
      .newSigner('fragJTOTestSigner2', 100n * 1_000_000_000n)
      .then(async (signer) => {
        await Promise.all([
          validator.airdropToken(
            signer.address,
            'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
            1_000n * AMOUNT_PER_JTO
          ),
        ]);
        return signer;
      }),
    validator
      .newSigner('fragJTOTestSigner3', 100n * 1_000_000_000n)
      .then(async (signer) => {
        await Promise.all([
          validator.airdropToken(
            signer.address,
            'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
            1_000n * AMOUNT_PER_JTO
          ),
        ]);
        return signer;
      }),
    validator
      .newSigner('fragJTOTestSigner4', 100n * 1_000_000_000n)
      .then(async (signer) => {
        await Promise.all([
          validator.airdropToken(
            signer.address,
            'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
            1_000n * AMOUNT_PER_JTO
          ),
        ]);
        return signer;
      }),
  ]);

  const user1 = ctx.user(signer1);
  const user2 = ctx.user(signer2);
  const user3 = ctx.user(signer3);
  const user4 = ctx.user(signer4);

  /** 1. configuration **/
  test(`restaking.fragJTO initializationTasks snapshot`, async () => {
    await expectMasked(initializationTasks).resolves.toMatchSnapshot();
  });

  test(`restaking.fragJTO.resolve`, async () => {
    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "MASKED(__lookupTableAddress)",
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
        "depositResidualMicroReceiptTokenAmount": 0n,
        "metadata": null,
        "normalizedToken": null,
        "oneReceiptTokenAsSOL": 0n,
        "receiptTokenDecimals": 9,
        "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
        "receiptTokenSupply": 0n,
        "restakingVaultReceiptTokens": [
          {
            "mint": "FRJtoBLuU72X3qgkVeBU1wXtmgQpWQmWptYsAdyyu3qT",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "vault": "BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 9,
            "depositable": true,
            "mint": "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 14476690n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
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
            "distributingRewardTokens": [
              {
                "harvestThresholdIntervalSeconds": 0n,
                "harvestThresholdMaxAmount": 18446744073709551615n,
                "harvestThresholdMinAmount": 0n,
                "lastHarvestedAt": 0n,
                "mint": "REALSWTCH7J8JdafNBLZpfSCLiFwpMCqod2RpkU4RNn",
              },
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

  /** 2. basic contribution test **/
  test(`rewards are settled based on the contribution proportion`, async () => {
    // user1 deposits 100 JTO and get 100 fragJTO
    await expectMasked(
      user1.deposit.execute(
        {
          assetMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
          assetAmount: 100n * AMOUNT_PER_JTO,
        },
        { signers: [signer1] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": 100000000000n,
          "assetMint": "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
          "metadata": null,
        },
        "events": {
          "unknown": [],
          "userCreatedOrUpdatedFundAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "user": "FwbWe1Dm9yVJrBsWA8J5e364rJ3MZNKrv3yVyWqAge7B",
            "userFundAccount": "3JxD7S8V5ZueN7D6ds3CRM4WCsSWtFu1H6A5vu5L8ywb",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "user": "FwbWe1Dm9yVJrBsWA8J5e364rJ3MZNKrv3yVyWqAge7B",
            "userRewardAccount": "5tg3SiYZovwsMBurdBPPiWiexD2h4Yc65wwtV4PEzTTH",
          },
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 100000000000n,
            "fundAccount": "Ee1W9enx3w2zv3pkgyNSqWteCaNJwxXBLydDMdTdPUzC",
            "mintedReceiptTokenAmount": 100000000000n,
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
            },
            "updatedUserRewardAccounts": [
              "5tg3SiYZovwsMBurdBPPiWiexD2h4Yc65wwtV4PEzTTH",
            ],
            "user": "FwbWe1Dm9yVJrBsWA8J5e364rJ3MZNKrv3yVyWqAge7B",
            "userFundAccount": "3JxD7S8V5ZueN7D6ds3CRM4WCsSWtFu1H6A5vu5L8ywb",
            "userReceiptTokenAccount": "9AKepsFr9maA8gJ72hrYPHZHpeTYH3ZLgp9y9Qbn34iG",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "FCT3V2ZFVrnNaNoXz1yzKyCm2c9w9GrEbp7gdNhN9rVt",
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

    // check user1's initial contribution
    let currentUser1RewardAccount = await user1.reward.resolve(true);
    const user1Slot0 = currentUser1RewardAccount!.basePool.updatedSlot; // starting slot
    expect(currentUser1RewardAccount!.basePool.contribution).toEqual(0n);
    expect(currentUser1RewardAccount!.bonusPool.contribution).toEqual(0n);

    // *** 100 slot elapsed ***
    await validator.skipSlots(100n);

    // user2 deposits 200 JTO and get 200 fragJTO
    await expectMasked(
      user2.deposit.execute(
        {
          assetMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
          assetAmount: 200n * AMOUNT_PER_JTO,
        },
        { signers: [signer2] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": 200000000000n,
          "assetMint": "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
          "metadata": null,
        },
        "events": {
          "unknown": [],
          "userCreatedOrUpdatedFundAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "user": "FRBRUhd8Tv5ZiUB5WYg9jdAdH5btZeRMfcXagxGLtwqQ",
            "userFundAccount": "DbRVUoZaXhDftjsA3vi92Fb2zg59RBFKeShC32JUY4V4",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "user": "FRBRUhd8Tv5ZiUB5WYg9jdAdH5btZeRMfcXagxGLtwqQ",
            "userRewardAccount": "J4HxfL4xvUFJ7EN6nkK88NMVUQhd3xcSRHGpnb9B4YP8",
          },
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 200000000000n,
            "fundAccount": "Ee1W9enx3w2zv3pkgyNSqWteCaNJwxXBLydDMdTdPUzC",
            "mintedReceiptTokenAmount": 200000000000n,
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
            },
            "updatedUserRewardAccounts": [
              "J4HxfL4xvUFJ7EN6nkK88NMVUQhd3xcSRHGpnb9B4YP8",
            ],
            "user": "FRBRUhd8Tv5ZiUB5WYg9jdAdH5btZeRMfcXagxGLtwqQ",
            "userFundAccount": "DbRVUoZaXhDftjsA3vi92Fb2zg59RBFKeShC32JUY4V4",
            "userReceiptTokenAccount": "DXKrvnXc37tLuhSjPrp2vcbV7RuUxtSG4KGVyU5gbh7U",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "DrqNrnpPJr9kdid5dPxwEbsbnUPDSZoFz9fiJG4iRyqn",
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

    // check user2's initial contribution
    let currentUser2RewardAccount = await user2.reward.resolve(true);
    const user2Slot100 = currentUser2RewardAccount!.basePool.updatedSlot; // 100 slot elapsed from slot0
    expect(currentUser2RewardAccount!.basePool.contribution).toEqual(0n);
    expect(currentUser2RewardAccount!.bonusPool.contribution).toEqual(0n);

    // *** 100 slot elapsed ***
    await validator.skipSlots(100n);

    // user1 deposits 300 JTO and get 300 fragJTO
    await expectMasked(
      user1.deposit.execute(
        {
          assetMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
          assetAmount: 300n * AMOUNT_PER_JTO,
        },
        { signers: [signer1] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": 300000000000n,
          "assetMint": "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
          "metadata": null,
        },
        "events": {
          "unknown": [],
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 300000000000n,
            "fundAccount": "Ee1W9enx3w2zv3pkgyNSqWteCaNJwxXBLydDMdTdPUzC",
            "mintedReceiptTokenAmount": 300000000000n,
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
            },
            "updatedUserRewardAccounts": [
              "5tg3SiYZovwsMBurdBPPiWiexD2h4Yc65wwtV4PEzTTH",
            ],
            "user": "FwbWe1Dm9yVJrBsWA8J5e364rJ3MZNKrv3yVyWqAge7B",
            "userFundAccount": "3JxD7S8V5ZueN7D6ds3CRM4WCsSWtFu1H6A5vu5L8ywb",
            "userReceiptTokenAccount": "9AKepsFr9maA8gJ72hrYPHZHpeTYH3ZLgp9y9Qbn34iG",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "FCT3V2ZFVrnNaNoXz1yzKyCm2c9w9GrEbp7gdNhN9rVt",
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
    // user2 updates reward pool
    await user2.reward.updatePools.execute(null);

    const user1Slot200 = await user1.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.basePool.updatedSlot); // 200 slot elapsed from slot0
    const user2Slot200 = await user2.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.basePool.updatedSlot); // 200 slot elapsed from slot0

    // check user1's contribution
    currentUser1RewardAccount = await user1.reward.resolve(true);
    let currentContributionOfUser1 =
      currentUser1RewardAccount!.basePool.contribution;
    expect(currentContributionOfUser1).toEqual(
      100n *
        (user1Slot200 - user1Slot0) *
        BASIC_ACCRUAL_RATE *
        AMOUNT_PER_FRAGJTO
    );
    currentContributionOfUser1 =
      currentUser1RewardAccount!.bonusPool.contribution;
    expect(currentContributionOfUser1).toEqual(
      100n *
        (user1Slot200 - user1Slot0) *
        BASIC_ACCRUAL_RATE *
        AMOUNT_PER_FRAGJTO
    );

    // check user2's contribution
    currentUser2RewardAccount = await user2.reward.resolve(true);
    let currentContributionOfUser2 =
      currentUser2RewardAccount!.basePool.contribution;
    expect(currentContributionOfUser2).toEqual(
      200n *
        (user2Slot200 - user2Slot100) *
        BASIC_ACCRUAL_RATE *
        AMOUNT_PER_FRAGJTO
    );
    currentContributionOfUser2 =
      currentUser2RewardAccount!.bonusPool.contribution;
    expect(currentContributionOfUser2).toEqual(
      200n *
        (user2Slot200 - user2Slot100) *
        BASIC_ACCRUAL_RATE *
        AMOUNT_PER_FRAGJTO
    );

    // *** 100 slot elapsed ***
    await validator.skipSlots(100n);
    await user1.reward.updatePools.execute(null);
    await user2.reward.updatePools.execute(null);
    const user1Slot300 = await user1.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.basePool.updatedSlot); // 300 slot elapsed from the start
    const user2Slot300 = await user2.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.basePool.updatedSlot); // 300 slot elapsed from the start

    // check user1's contribution
    currentUser1RewardAccount = await user1.reward.resolve(true);
    currentContributionOfUser1 =
      currentUser1RewardAccount!.basePool.contribution;
    expect(currentContributionOfUser1).toEqual(
      100n *
        (user1Slot300 - user1Slot0) *
        BASIC_ACCRUAL_RATE *
        AMOUNT_PER_FRAGJTO +
        300n *
          (user1Slot300 - user1Slot200) *
          BASIC_ACCRUAL_RATE *
          AMOUNT_PER_FRAGJTO
    );
    currentContributionOfUser1 =
      currentUser1RewardAccount!.bonusPool.contribution;
    expect(currentContributionOfUser1).toEqual(
      100n *
        (user1Slot300 - user1Slot0) *
        BASIC_ACCRUAL_RATE *
        AMOUNT_PER_FRAGJTO +
        300n *
          (user1Slot300 - user1Slot200) *
          BASIC_ACCRUAL_RATE *
          AMOUNT_PER_FRAGJTO
    );

    // check user2's contribution
    currentUser2RewardAccount = await user2.reward.resolve(true);
    currentContributionOfUser2 =
      currentUser2RewardAccount!.basePool.contribution;
    expect(currentContributionOfUser2).toEqual(
      200n *
        (user2Slot300 - user2Slot100) *
        BASIC_ACCRUAL_RATE *
        AMOUNT_PER_FRAGJTO
    );
    currentContributionOfUser2 =
      currentUser2RewardAccount!.bonusPool.contribution;
    expect(currentContributionOfUser2).toEqual(
      200n *
        (user2Slot300 - user2Slot100) *
        BASIC_ACCRUAL_RATE *
        AMOUNT_PER_FRAGJTO
    );

    // drop fPoint in approximately(time flies) 1:1 ratio to total contribution. contribution(11) has 2 + 5 more decimals than fPoint(4)
    await restaking.fragJTO.reward.updatePools.execute(null);
    const amountToSettle =
      (await restaking.fragJTO.reward
        .resolve(true)
        .then((rewardAccount) => rewardAccount!.bonusPool.contribution)) /
      10_000_000n;
    await restaking.fragJTO.reward.settleReward.execute({
      isBonus: true,
      mint: '11111111111111111111111111111111',
      amount: amountToSettle,
    });

    const rewardSettlement = await restaking.fragJTO.reward
      .resolve(true)
      .then((rewardAccount) => rewardAccount!.bonusPool.settlements[0]);
    const settledAmount =
      rewardSettlement.blocks[rewardSettlement.blocks.length - 1].amount;
    expect(amountToSettle).toEqual(settledAmount);

    // *** 100 slot elapsed ***
    await validator.skipSlots(100n);
    await user1.reward.updatePools.execute(null);
    await user2.reward.updatePools.execute(null);

    currentUser1RewardAccount = await user1.reward.resolve(true);
    const user1SettledAmount =
      currentUser1RewardAccount!.bonusPool.settlements[0].settledAmount;
    const user1SettledContribution =
      currentUser1RewardAccount!.bonusPool.settlements[0].settledContribution;

    currentUser2RewardAccount = await user2.reward.resolve(true);
    const user2SettledAmount =
      currentUser2RewardAccount!.bonusPool.settlements[0].settledAmount;
    const user2SettledContribution =
      currentUser2RewardAccount!.bonusPool.settlements[0].settledContribution;

    const ratio1 =
      Number(user1SettledAmount) / Number(user1SettledContribution);
    const ratio2 =
      Number(user2SettledAmount) / Number(user2SettledContribution);

    expect(Math.abs(ratio1 - ratio2)).toBeLessThanOrEqual(
      Number.EPSILON * 10000
    );
  });

  /** 3. custom accrual rate test */
  test(`rewards can be settled with custom contribution accrual rate enabled`, async () => {
    // starts with user1: 400 fragJTO, user2: 200 fragJTO
    await expectMasked(
      user2.deposit.execute(
        {
          assetMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
          assetAmount: 200n * AMOUNT_PER_JTO,
          metadata: {
            user: user2.address!.toString(),
            walletProvider: 'STIMPACK',
            contributionAccrualRate: 150,
            expiredAt: new Date('9999-01-01T00:00:00Z'),
            signerKeyPair: adminSigner.keyPair,
          },
        },
        { signers: [signer2] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": 200000000000n,
          "assetMint": "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
          "metadata": {
            "contributionAccrualRate": 150,
            "expiredAt": "MASKED(/.*At?$/)",
            "signerKeyPair": {
              "privateKey": {},
              "publicKey": {},
            },
            "user": "FRBRUhd8Tv5ZiUB5WYg9jdAdH5btZeRMfcXagxGLtwqQ",
            "walletProvider": "STIMPACK",
          },
        },
        "events": {
          "unknown": [],
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "Some",
              "value": 150,
            },
            "depositedAmount": 200000000000n,
            "fundAccount": "Ee1W9enx3w2zv3pkgyNSqWteCaNJwxXBLydDMdTdPUzC",
            "mintedReceiptTokenAmount": 200000000000n,
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
            },
            "updatedUserRewardAccounts": [
              "J4HxfL4xvUFJ7EN6nkK88NMVUQhd3xcSRHGpnb9B4YP8",
            ],
            "user": "FRBRUhd8Tv5ZiUB5WYg9jdAdH5btZeRMfcXagxGLtwqQ",
            "userFundAccount": "DbRVUoZaXhDftjsA3vi92Fb2zg59RBFKeShC32JUY4V4",
            "userReceiptTokenAccount": "DXKrvnXc37tLuhSjPrp2vcbV7RuUxtSG4KGVyU5gbh7U",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "DrqNrnpPJr9kdid5dPxwEbsbnUPDSZoFz9fiJG4iRyqn",
            },
            "walletProvider": {
              "__option": "Some",
              "value": "STIMPACK",
            },
          },
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);

    // flush contributions of all pools by settling zero rewards
    await expectMasked(
      restaking.fragJTO.reward.settleReward.execute({
        isBonus: false,
        mint: '11111111111111111111111111111111',
        amount: 0n,
      })
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "amount": 0n,
          "isBonus": false,
          "mint": "11111111111111111111111111111111",
        },
        "events": {
          "fundManagerUpdatedRewardPool": {
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "rewardAccount": "EfWLuf9Wmk4XKSLvvAvTHH7M3z8airf1zwdNbMEP5dP9",
          },
          "unknown": [],
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);
    await expectMasked(
      restaking.fragJTO.reward.settleReward.execute({
        isBonus: true,
        mint: '11111111111111111111111111111111',
        amount: 0n,
      })
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "amount": 0n,
          "isBonus": true,
          "mint": "11111111111111111111111111111111",
        },
        "events": {
          "fundManagerUpdatedRewardPool": {
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "rewardAccount": "EfWLuf9Wmk4XKSLvvAvTHH7M3z8airf1zwdNbMEP5dP9",
          },
          "unknown": [],
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);

    // *** 100 slot elapsed ***
    await validator.skipSlots(100n);
    await user1.reward.updatePools.execute(null);
    await user2.reward.updatePools.execute(null);
    const user1PrevBonusPool = await user1.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.bonusPool);
    const user2PrevBonusPool = await user2.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.bonusPool);
    const user1Slot500 = user1PrevBonusPool.updatedSlot; // 500 slot elapsed from slot0
    const user2Slot500 = user2PrevBonusPool.updatedSlot; // 500 slot elapsed from slot0

    // drop fPoint in approximately(time flies) 2:1 ratio to total contribution; contribution(11) has 2 + 5 more decimals than fPoint(4)
    await restaking.fragJTO.reward.updatePools.execute(null);
    const amountToSettle =
      ((await restaking.fragJTO.reward
        .resolve(true)
        .then((rewardAccount) => rewardAccount!.bonusPool.contribution)) *
        200n) /
      10_000_000n;

    await restaking.fragJTO.reward.settleReward.execute({
      isBonus: false,
      mint: '11111111111111111111111111111111',
      amount: amountToSettle,
    });
    await restaking.fragJTO.reward.settleReward.execute({
      isBonus: true,
      mint: '11111111111111111111111111111111',
      amount: amountToSettle,
    });
    await restaking.fragJTO.reward.updatePools.execute(null);

    const rewardBasePool = await restaking.fragJTO.reward
      .resolve(true)
      .then((rewardAccount) => rewardAccount!.basePool);
    const rewardBonusPool = await restaking.fragJTO.reward
      .resolve(true)
      .then((rewardAccount) => rewardAccount!.bonusPool);

    expect(rewardBasePool.updatedSlot).toEqual(rewardBonusPool.updatedSlot);
    expect(rewardBasePool.tokenAllocatedAmount.totalAmount).toEqual(
      rewardBonusPool.tokenAllocatedAmount.totalAmount
    );
    expect(rewardBasePool.contribution).toBeLessThan(
      rewardBonusPool.contribution
    );
    expect(
      rewardBasePool.settlements[0].settlementBlocksLastRewardPoolContribution
    ).toBeLessThan(
      rewardBonusPool.settlements[0].settlementBlocksLastRewardPoolContribution
    );

    // now check users' settlements
    await user1.reward.updatePools.execute(null);
    await user2.reward.updatePools.execute(null);

    // new base pool settled amounts are same; A: 400, B: 400 => A:B = 1:1
    const user1BasePool = await user1.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.basePool);
    const user2BasePool = await user2.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.basePool);
    const user1UpdatedSlot = user1BasePool.updatedSlot;
    const user2UpdatedSlot = user2BasePool.updatedSlot;

    const user1BasePoolTotalSettledAmount =
      user1BasePool.settlements[0].settledAmount;
    const user2BasePoolTotalSettledAmount =
      user2BasePool.settlements[0].settledAmount;
    expect(user1BasePoolTotalSettledAmount).toEqual(
      user2BasePoolTotalSettledAmount
    );

    const user1BasePoolTokenAllocatedAmount =
      user1BasePool.tokenAllocatedAmount;
    const user2BasePoolTokenAllocatedAmount =
      user2BasePool.tokenAllocatedAmount;
    expect(user1BasePoolTokenAllocatedAmount).toEqual(
      user2BasePoolTokenAllocatedAmount
    );

    // added bonus pool settled amount are different; A: 400, B: 200 + 200(x1.5) => A:B = 4:5
    const user1BonusPool = await user1.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.bonusPool);
    const user2BonusPool = await user2.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.bonusPool);

    const user1BonusSettledAmountDelta =
      user1BonusPool.settlements[0].settledAmount -
      user1PrevBonusPool.settlements[0].settledAmount;
    const user2BonusSettledAmountDelta =
      user2BonusPool.settlements[0].settledAmount -
      user2PrevBonusPool.settlements[0].settledAmount;
    const user1BonusSettledContributionDelta =
      user1BonusPool.settlements[0].settledContribution -
      user1PrevBonusPool.settlements[0].settledContribution;
    const user2BonusSettledContributionDelta =
      user2BonusPool.settlements[0].settledContribution -
      user2PrevBonusPool.settlements[0].settledContribution;

    const amountRatio =
      Number(user1BonusSettledAmountDelta) /
      Number(user2BonusSettledAmountDelta);
    const contributionRatio =
      Number(user1BonusSettledContributionDelta) /
      Number(user2BonusSettledContributionDelta);
    expect(Math.abs(amountRatio - contributionRatio)).toBeLessThanOrEqual(
      Number.EPSILON * 100000
    );

    // user1 contribution * 5 == user2 contribution * 4
    expect(user1BonusSettledContributionDelta * 5n).toEqual(
      user2BonusSettledContributionDelta * 4n
    );
  });

  /** 4. contribution test with token transfer (user3 has user_reward_account, user4 doesn't have user_reward_account) **/
  test(`contribution is accumulated with users who have user_reward_account`, async () => {
    await expectMasked(
      user3.deposit.execute(
        {
          assetMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
          assetAmount: 100n * AMOUNT_PER_JTO,
        },
        { signers: [signer3] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": 100000000000n,
          "assetMint": "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
          "metadata": null,
        },
        "events": {
          "unknown": [],
          "userCreatedOrUpdatedFundAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "user": "DgMNmDjkhXvgtw5rKmcwGEUbcXRsNhnhJGnrgNhDVJTr",
            "userFundAccount": "DZT4jRNY2YnWXq82KHRkTS2qAkEardHmaMGDbveXqxsN",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "user": "DgMNmDjkhXvgtw5rKmcwGEUbcXRsNhnhJGnrgNhDVJTr",
            "userRewardAccount": "Dj1oZ3Sgwv8DWhSoNhhN5vSsNNSMyFvKvg6d8ecRDtr9",
          },
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 100000000000n,
            "fundAccount": "Ee1W9enx3w2zv3pkgyNSqWteCaNJwxXBLydDMdTdPUzC",
            "mintedReceiptTokenAmount": 100000000000n,
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
            },
            "updatedUserRewardAccounts": [
              "Dj1oZ3Sgwv8DWhSoNhhN5vSsNNSMyFvKvg6d8ecRDtr9",
            ],
            "user": "DgMNmDjkhXvgtw5rKmcwGEUbcXRsNhnhJGnrgNhDVJTr",
            "userFundAccount": "DZT4jRNY2YnWXq82KHRkTS2qAkEardHmaMGDbveXqxsN",
            "userReceiptTokenAccount": "DfoK2kVKtMYyydzQwN9de8ikW5vbf3m97TMq4XesjtAE",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "7iZqHGJmARhzDCxVwGYEtzdiuZamLNmuR5cKXL9zA6oR",
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
    await validator.skipSlots(100n);

    // user3 transfers 100 FRAGJTO to user4
    await expectMasked(
      user3.transfer.execute(
        {
          receiptTokenAmount: 100n * AMOUNT_PER_FRAGJTO,
          recipient: await user4
            .resolveAddress(true)
            .then((address) => address!.toString()),
        },
        { signers: [signer3] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "receiptTokenAmount": 100000000000n,
          "recipient": "HFfieEFm71E1FrU7JEJbHMihh95wwyxBFTStuU8wUmie",
        },
        "events": {
          "unknown": [],
          "userTransferredReceiptToken": {
            "destination": "HFfieEFm71E1FrU7JEJbHMihh95wwyxBFTStuU8wUmie",
            "destinationFundAccount": {
              "__option": "None",
            },
            "destinationReceiptTokenAccount": "Ea1o6kMwuYDVqXHb3zTgu7bxbqKMuh9Qt4LZ7xJFT4Fb",
            "fundAccount": "Ee1W9enx3w2zv3pkgyNSqWteCaNJwxXBLydDMdTdPUzC",
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "source": "DgMNmDjkhXvgtw5rKmcwGEUbcXRsNhnhJGnrgNhDVJTr",
            "sourceFundAccount": {
              "__option": "Some",
              "value": "DZT4jRNY2YnWXq82KHRkTS2qAkEardHmaMGDbveXqxsN",
            },
            "sourceReceiptTokenAccount": "DfoK2kVKtMYyydzQwN9de8ikW5vbf3m97TMq4XesjtAE",
            "transferredReceiptTokenAmount": 100000000000n,
            "updatedUserRewardAccounts": [
              "Dj1oZ3Sgwv8DWhSoNhhN5vSsNNSMyFvKvg6d8ecRDtr9",
            ],
          },
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);
    await restaking.fragJTO.reward.updatePools.execute(null);
    const rewardPoolAtSlot600 = await restaking.fragJTO.reward
      .resolve(true)
      .then((rewardAccount) => rewardAccount!.basePool);
    const rewardPoolSlot600 = rewardPoolAtSlot600.updatedSlot;
    await user3.reward.updatePools.execute(null);
    const user3RewardPoolAtSlot600 = await user3.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.basePool);
    const userSlot600 = user3RewardPoolAtSlot600.updatedSlot;

    await validator.skipSlots(100n);

    await restaking.fragJTO.reward.updatePools.execute(null);
    let rewardPoolAtSlot700 = await restaking.fragJTO.reward
      .resolve(true)
      .then((rewardAccount) => rewardAccount!.basePool);
    let rewardPoolSlot700 = rewardPoolAtSlot700.updatedSlot;
    await user3.reward.updatePools.execute(null);
    let user3RewardPoolAtSlot700 = await user3.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.basePool);
    let userSlot700 = user3RewardPoolAtSlot700.updatedSlot;

    expect(user3RewardPoolAtSlot700.contribution).toEqual(
      user3RewardPoolAtSlot600.contribution
    );
    expect(rewardPoolAtSlot700.contribution).toEqual(
      rewardPoolAtSlot600.contribution +
        rewardPoolAtSlot600.tokenAllocatedAmount.totalAmount *
          BASIC_ACCRUAL_RATE *
          (rewardPoolSlot700 - rewardPoolSlot600)
    );

    await expectMasked(
      user4.transfer.execute(
        {
          receiptTokenAmount: 100n * AMOUNT_PER_FRAGJTO,
          recipient: await user3
            .resolveAddress(true)
            .then((address) => address!.toString()),
        },
        { signers: [signer4] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "receiptTokenAmount": 100000000000n,
          "recipient": "DgMNmDjkhXvgtw5rKmcwGEUbcXRsNhnhJGnrgNhDVJTr",
        },
        "events": {
          "unknown": [],
          "userTransferredReceiptToken": {
            "destination": "DgMNmDjkhXvgtw5rKmcwGEUbcXRsNhnhJGnrgNhDVJTr",
            "destinationFundAccount": {
              "__option": "Some",
              "value": "DZT4jRNY2YnWXq82KHRkTS2qAkEardHmaMGDbveXqxsN",
            },
            "destinationReceiptTokenAccount": "DfoK2kVKtMYyydzQwN9de8ikW5vbf3m97TMq4XesjtAE",
            "fundAccount": "Ee1W9enx3w2zv3pkgyNSqWteCaNJwxXBLydDMdTdPUzC",
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "source": "HFfieEFm71E1FrU7JEJbHMihh95wwyxBFTStuU8wUmie",
            "sourceFundAccount": {
              "__option": "None",
            },
            "sourceReceiptTokenAccount": "Ea1o6kMwuYDVqXHb3zTgu7bxbqKMuh9Qt4LZ7xJFT4Fb",
            "transferredReceiptTokenAmount": 100000000000n,
            "updatedUserRewardAccounts": [
              "Dj1oZ3Sgwv8DWhSoNhhN5vSsNNSMyFvKvg6d8ecRDtr9",
            ],
          },
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);

    await restaking.fragJTO.reward.updatePools.execute(null);
    rewardPoolAtSlot700 = await restaking.fragJTO.reward
      .resolve(true)
      .then((rewardAccount) => rewardAccount!.basePool);
    rewardPoolSlot700 = rewardPoolAtSlot700.updatedSlot;
    await user3.reward.updatePools.execute(null);
    user3RewardPoolAtSlot700 = await user3.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.basePool);
    userSlot700 = user3RewardPoolAtSlot700.updatedSlot;

    await validator.skipSlots(100n);

    await restaking.fragJTO.reward.updatePools.execute(null);
    const rewardPoolAtSlot800 = await restaking.fragJTO.reward
      .resolve(true)
      .then((rewardAccount) => rewardAccount!.basePool);
    const rewardPoolSlot800 = rewardPoolAtSlot800.updatedSlot;
    await user3.reward.updatePools.execute(null);
    const user3RewardPoolAtSlot800 = await user3.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.basePool);
    const userSlot800 = user3RewardPoolAtSlot800.updatedSlot;

    expect(
      user3RewardPoolAtSlot800.contribution -
        user3RewardPoolAtSlot700.contribution
    ).toEqual(
      100n *
        (userSlot800 - userSlot700) *
        BASIC_ACCRUAL_RATE *
        AMOUNT_PER_FRAGJTO
    );
  });

  /** 5. contribution test with token wrap & unwrap (user3 wraps & unwraps FRAGJTO) **/
  test(`wrapping FRAGXXX affects token allocated amount of user, but global reward account maintains same amount`, async () => {
    // user3 wraps 100 FRAGJTO
    await expectMasked(
      user3.wrap.execute(
        {
          receiptTokenAmount: 100n * AMOUNT_PER_FRAGJTO,
        },
        { signers: [signer3] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "receiptTokenAmount": 100000000000n,
          "receiptTokenAmountAsTargetBalance": false,
        },
        "events": {
          "unknown": [],
          "userWrappedReceiptToken": {
            "fundAccount": "Ee1W9enx3w2zv3pkgyNSqWteCaNJwxXBLydDMdTdPUzC",
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "updatedFundWrapAccountRewardAccount": "EyEb3k5uaknwX47tSyzhBLS9DnUGrfPv16uamEXJmeaB",
            "updatedUserFundAccount": {
              "__option": "Some",
              "value": "DZT4jRNY2YnWXq82KHRkTS2qAkEardHmaMGDbveXqxsN",
            },
            "updatedUserRewardAccount": {
              "__option": "Some",
              "value": "Dj1oZ3Sgwv8DWhSoNhhN5vSsNNSMyFvKvg6d8ecRDtr9",
            },
            "user": "DgMNmDjkhXvgtw5rKmcwGEUbcXRsNhnhJGnrgNhDVJTr",
            "userReceiptTokenAccount": "DfoK2kVKtMYyydzQwN9de8ikW5vbf3m97TMq4XesjtAE",
            "userWrappedTokenAccount": "2fHtBgC4RkQxrXGxDv5btjeWMSjc6iBRPK9WsXVRkWBL",
            "wrappedReceiptTokenAmount": 100000000000n,
            "wrappedTokenMint": "EAvS1wFjAccNpDYbAkW2dwUDEiC7BMvWzwUj2tjRUkHA",
          },
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);

    await restaking.fragJTO.reward.updatePools.execute(null);
    const rewardPoolAtSlot800 = await restaking.fragJTO.reward
      .resolve(true)
      .then((rewardAccount) => rewardAccount!.basePool);
    const rewardPoolSlot800 = rewardPoolAtSlot800.updatedSlot;
    await user3.reward.updatePools.execute(null);
    const user3RewardPoolAtSlot800 = await user3.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.basePool);
    const userSlot800 = user3RewardPoolAtSlot800.updatedSlot;

    await validator.skipSlots(100n);

    await restaking.fragJTO.reward.updatePools.execute(null);
    let rewardPoolAtSlot900 = await restaking.fragJTO.reward
      .resolve(true)
      .then((rewardAccount) => rewardAccount!.basePool);
    let rewardPoolSlot900 = rewardPoolAtSlot900.updatedSlot;
    await user3.reward.updatePools.execute(null);
    let user3RewardPoolAtSlot900 = await user3.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.basePool);
    let userSlot900 = user3RewardPoolAtSlot900.updatedSlot;

    expect(user3RewardPoolAtSlot900.tokenAllocatedAmount.totalAmount).toEqual(
      0n
    );
    expect(user3RewardPoolAtSlot900.contribution).toEqual(
      user3RewardPoolAtSlot800.contribution
    );
    expect(rewardPoolAtSlot900.tokenAllocatedAmount.totalAmount).toEqual(
      rewardPoolAtSlot800.tokenAllocatedAmount.totalAmount
    );

    // user3 unwraps 100 wFRAGJTO
    await expectMasked(
      user3.unwrap.execute(
        {
          wrappedTokenAmount: 100n * AMOUNT_PER_WFRAGJTO,
        },
        { signers: [signer3] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "wrappedTokenAmount": 100000000000n,
        },
        "events": {
          "unknown": [],
          "userUnwrappedReceiptToken": {
            "fundAccount": "Ee1W9enx3w2zv3pkgyNSqWteCaNJwxXBLydDMdTdPUzC",
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "unwrappedReceiptTokenAmount": 100000000000n,
            "updatedFundWrapAccountRewardAccount": "EyEb3k5uaknwX47tSyzhBLS9DnUGrfPv16uamEXJmeaB",
            "updatedUserFundAccount": {
              "__option": "Some",
              "value": "DZT4jRNY2YnWXq82KHRkTS2qAkEardHmaMGDbveXqxsN",
            },
            "updatedUserRewardAccount": {
              "__option": "Some",
              "value": "Dj1oZ3Sgwv8DWhSoNhhN5vSsNNSMyFvKvg6d8ecRDtr9",
            },
            "user": "DgMNmDjkhXvgtw5rKmcwGEUbcXRsNhnhJGnrgNhDVJTr",
            "userReceiptTokenAccount": "DfoK2kVKtMYyydzQwN9de8ikW5vbf3m97TMq4XesjtAE",
            "userWrappedTokenAccount": "2fHtBgC4RkQxrXGxDv5btjeWMSjc6iBRPK9WsXVRkWBL",
            "wrappedTokenMint": "EAvS1wFjAccNpDYbAkW2dwUDEiC7BMvWzwUj2tjRUkHA",
          },
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);

    await restaking.fragJTO.reward.updatePools.execute(null);
    rewardPoolAtSlot900 = await restaking.fragJTO.reward
      .resolve(true)
      .then((rewardAccount) => rewardAccount!.basePool);
    rewardPoolSlot900 = rewardPoolAtSlot900.updatedSlot;
    await user3.reward.updatePools.execute(null);
    user3RewardPoolAtSlot900 = await user3.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.basePool);
    userSlot900 = user3RewardPoolAtSlot900.updatedSlot;

    await validator.skipSlots(100n);

    await restaking.fragJTO.reward.updatePools.execute(null);
    const rewardPoolAtSlot1000 = await restaking.fragJTO.reward
      .resolve(true)
      .then((rewardAccount) => rewardAccount!.basePool);
    const rewardPoolSlot1000 = rewardPoolAtSlot1000.updatedSlot;
    await user3.reward.updatePools.execute(null);
    const user3RewardPoolAtSlot1000 = await user3.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.basePool);

    expect(user3RewardPoolAtSlot1000.tokenAllocatedAmount.totalAmount).toEqual(
      100n * AMOUNT_PER_FRAGJTO
    );
  });

  /** 6. token is subtracted from user account in ascending order (contribution accural rate low to high **/
  test(`record with low contribution rate is deleted first`, async () => {
    // user4 deposits 200 JTO with 150 accrual rate enabled
    await expectMasked(
      user4.deposit.execute(
        {
          assetMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
          assetAmount: 200n * AMOUNT_PER_JTO,
          metadata: {
            user: user4.address!.toString(),
            walletProvider: 'TERRY',
            contributionAccrualRate: 150,
            expiredAt: new Date('9999-01-01T00:00:00Z'),
            signerKeyPair: adminSigner.keyPair,
          },
        },
        { signers: [signer4] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": 200000000000n,
          "assetMint": "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
          "metadata": {
            "contributionAccrualRate": 150,
            "expiredAt": "MASKED(/.*At?$/)",
            "signerKeyPair": {
              "privateKey": {},
              "publicKey": {},
            },
            "user": "HFfieEFm71E1FrU7JEJbHMihh95wwyxBFTStuU8wUmie",
            "walletProvider": "TERRY",
          },
        },
        "events": {
          "unknown": [],
          "userCreatedOrUpdatedFundAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "user": "HFfieEFm71E1FrU7JEJbHMihh95wwyxBFTStuU8wUmie",
            "userFundAccount": "9NAcdxEDravoYALBpnDBvzUk6YwKsce72qpoVDEjK7BE",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "user": "HFfieEFm71E1FrU7JEJbHMihh95wwyxBFTStuU8wUmie",
            "userRewardAccount": "F2UFLhATDQJu8sKz7ecMQ5gyr7GR3VuoCjXSLPiHDa1p",
          },
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "Some",
              "value": 150,
            },
            "depositedAmount": 200000000000n,
            "fundAccount": "Ee1W9enx3w2zv3pkgyNSqWteCaNJwxXBLydDMdTdPUzC",
            "mintedReceiptTokenAmount": 200000000000n,
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
            },
            "updatedUserRewardAccounts": [
              "F2UFLhATDQJu8sKz7ecMQ5gyr7GR3VuoCjXSLPiHDa1p",
            ],
            "user": "HFfieEFm71E1FrU7JEJbHMihh95wwyxBFTStuU8wUmie",
            "userFundAccount": "9NAcdxEDravoYALBpnDBvzUk6YwKsce72qpoVDEjK7BE",
            "userReceiptTokenAccount": "Ea1o6kMwuYDVqXHb3zTgu7bxbqKMuh9Qt4LZ7xJFT4Fb",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "49GyoPBtg3rfx93vjD2Ye52ATCfLB4YhcMChT4yV3HQf",
            },
            "walletProvider": {
              "__option": "Some",
              "value": "TERRY",
            },
          },
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);
    const user4SlotAfterDeposit = await user4.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.bonusPool.updatedSlot);

    // user4 gets 100 FRAGJTO from user3.
    // user4 now has 300 FRAGJTO - 200FRAGJTO(150 accrual rate) + 100FRAGJTO(100 accrual rate))
    await expectMasked(
      user3.transfer.execute(
        {
          receiptTokenAmount: 100n * AMOUNT_PER_FRAGJTO,
          recipient: user4.address!.toString(),
        },
        { signers: [signer3] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "receiptTokenAmount": 100000000000n,
          "recipient": "HFfieEFm71E1FrU7JEJbHMihh95wwyxBFTStuU8wUmie",
        },
        "events": {
          "unknown": [],
          "userTransferredReceiptToken": {
            "destination": "HFfieEFm71E1FrU7JEJbHMihh95wwyxBFTStuU8wUmie",
            "destinationFundAccount": {
              "__option": "Some",
              "value": "9NAcdxEDravoYALBpnDBvzUk6YwKsce72qpoVDEjK7BE",
            },
            "destinationReceiptTokenAccount": "Ea1o6kMwuYDVqXHb3zTgu7bxbqKMuh9Qt4LZ7xJFT4Fb",
            "fundAccount": "Ee1W9enx3w2zv3pkgyNSqWteCaNJwxXBLydDMdTdPUzC",
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "source": "DgMNmDjkhXvgtw5rKmcwGEUbcXRsNhnhJGnrgNhDVJTr",
            "sourceFundAccount": {
              "__option": "Some",
              "value": "DZT4jRNY2YnWXq82KHRkTS2qAkEardHmaMGDbveXqxsN",
            },
            "sourceReceiptTokenAccount": "DfoK2kVKtMYyydzQwN9de8ikW5vbf3m97TMq4XesjtAE",
            "transferredReceiptTokenAmount": 100000000000n,
            "updatedUserRewardAccounts": [
              "Dj1oZ3Sgwv8DWhSoNhhN5vSsNNSMyFvKvg6d8ecRDtr9",
              "F2UFLhATDQJu8sKz7ecMQ5gyr7GR3VuoCjXSLPiHDa1p",
            ],
          },
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);
    const user4SlotAfterTransfer = await user4.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.bonusPool.updatedSlot);

    await user4.reward.updatePools.execute(null);
    await restaking.fragJTO.reward.updatePools.execute(null);
    const user4RewardPoolAtSlot1000 = await user4.reward
      .resolve(true)
      .then((userRewardAccount) => userRewardAccount!.basePool);

    await validator.skipSlots(100n);

    await user4.reward.updatePools.execute(null);
    await restaking.fragJTO.reward.updatePools.execute(null);
    const user4RewardAccountAtSlot1100 = await user4.reward.resolve(true);
    const user4RewardPoolAtSlot1100 = user4RewardAccountAtSlot1100!.basePool;
    const user4Slot1100 = user4RewardPoolAtSlot1100.updatedSlot;

    expect(user4RewardAccountAtSlot1100!.bonusPool.contribution).toEqual(
      200n *
        (user4Slot1100 - user4SlotAfterDeposit) *
        150n *
        AMOUNT_PER_FRAGJTO +
        100n *
          (user4Slot1100 - user4SlotAfterTransfer) *
          BASIC_ACCRUAL_RATE *
          AMOUNT_PER_FRAGJTO
    );

    // user4 deposits 100 JTO with 130 accrual rate enabled
    await expectMasked(
      user4.deposit.execute(
        {
          assetMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
          assetAmount: 100n * AMOUNT_PER_JTO,
          metadata: {
            user: user4.address!.toString(),
            walletProvider: 'BACKPACK',
            contributionAccrualRate: 130,
            expiredAt: new Date('9999-01-01T00:00:00Z'),
            signerKeyPair: adminSigner.keyPair,
          },
        },
        { signers: [signer4] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": 100000000000n,
          "assetMint": "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
          "metadata": {
            "contributionAccrualRate": 130,
            "expiredAt": "MASKED(/.*At?$/)",
            "signerKeyPair": {
              "privateKey": {},
              "publicKey": {},
            },
            "user": "HFfieEFm71E1FrU7JEJbHMihh95wwyxBFTStuU8wUmie",
            "walletProvider": "BACKPACK",
          },
        },
        "events": {
          "unknown": [],
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "Some",
              "value": 130,
            },
            "depositedAmount": 100000000000n,
            "fundAccount": "Ee1W9enx3w2zv3pkgyNSqWteCaNJwxXBLydDMdTdPUzC",
            "mintedReceiptTokenAmount": 100000000000n,
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
            },
            "updatedUserRewardAccounts": [
              "F2UFLhATDQJu8sKz7ecMQ5gyr7GR3VuoCjXSLPiHDa1p",
            ],
            "user": "HFfieEFm71E1FrU7JEJbHMihh95wwyxBFTStuU8wUmie",
            "userFundAccount": "9NAcdxEDravoYALBpnDBvzUk6YwKsce72qpoVDEjK7BE",
            "userReceiptTokenAccount": "Ea1o6kMwuYDVqXHb3zTgu7bxbqKMuh9Qt4LZ7xJFT4Fb",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "49GyoPBtg3rfx93vjD2Ye52ATCfLB4YhcMChT4yV3HQf",
            },
            "walletProvider": {
              "__option": "Some",
              "value": "BACKPACK",
            },
          },
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);

    // user4 transfers 250 FRAGJTO to user3
    // 100*1.0, 100*1.3, 200*1.5 => 150*1.5 expected
    await expectMasked(
      user4.transfer.execute(
        {
          receiptTokenAmount: 250n * AMOUNT_PER_FRAGJTO,
          recipient: user3.address!.toString(),
        },
        { signers: [signer4] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "receiptTokenAmount": 250000000000n,
          "recipient": "DgMNmDjkhXvgtw5rKmcwGEUbcXRsNhnhJGnrgNhDVJTr",
        },
        "events": {
          "unknown": [],
          "userTransferredReceiptToken": {
            "destination": "DgMNmDjkhXvgtw5rKmcwGEUbcXRsNhnhJGnrgNhDVJTr",
            "destinationFundAccount": {
              "__option": "Some",
              "value": "DZT4jRNY2YnWXq82KHRkTS2qAkEardHmaMGDbveXqxsN",
            },
            "destinationReceiptTokenAccount": "DfoK2kVKtMYyydzQwN9de8ikW5vbf3m97TMq4XesjtAE",
            "fundAccount": "Ee1W9enx3w2zv3pkgyNSqWteCaNJwxXBLydDMdTdPUzC",
            "receiptTokenMint": "bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v",
            "source": "HFfieEFm71E1FrU7JEJbHMihh95wwyxBFTStuU8wUmie",
            "sourceFundAccount": {
              "__option": "Some",
              "value": "9NAcdxEDravoYALBpnDBvzUk6YwKsce72qpoVDEjK7BE",
            },
            "sourceReceiptTokenAccount": "Ea1o6kMwuYDVqXHb3zTgu7bxbqKMuh9Qt4LZ7xJFT4Fb",
            "transferredReceiptTokenAmount": 250000000000n,
            "updatedUserRewardAccounts": [
              "F2UFLhATDQJu8sKz7ecMQ5gyr7GR3VuoCjXSLPiHDa1p",
              "Dj1oZ3Sgwv8DWhSoNhhN5vSsNNSMyFvKvg6d8ecRDtr9",
            ],
          },
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);
    await user4.reward.updatePools.execute(null);

    await expectMasked(
      user4.reward
        .resolve(true)
        .then((userRewardAccount) => userRewardAccount?.bonusPool)
    ).resolves.toMatchInlineSnapshot(`
      {
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
              "amount": 0n,
              "contributionAccrualRate": 1,
            },
            {
              "amount": 150000000000n,
              "contributionAccrualRate": 1.5,
            },
          ],
          "totalAmount": 150000000000n,
        },
        "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
      }
    `);
  });
});
