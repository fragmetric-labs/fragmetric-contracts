import { afterAll, beforeAll, describe, expect, test } from 'vitest';
import { createTestSuiteContext, expectMasked } from '../../testutil';
import { initializeFragSOL } from './fragsol.init';

describe('restaking.fragSOL test', async () => {
  const testCtx = initializeFragSOL(
    await createTestSuiteContext({ programs: { solv: false } })
  );

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  const { validator, feePayer, restaking, initializationTasks } = testCtx;
  const ctx = restaking.fragSOL;

  const [signer1] = await Promise.all([
    validator
      .newSigner('fragSOLDepositTestSigner1', 100_000_000_000n)
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

  /** 1. configuration **/
  test(`restaking.fragSOL initializationTasks snapshot`, async () => {
    await expectMasked(initializationTasks).resolves.toMatchSnapshot();
  });

  test('restaking.fragSOL.resolve', async () => {
    await expect(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "G45gQa12Uwvnrp2Yb9oWTSwZSEHZWL71QDWvyLz23bNc",
        "__pricingSources": [
          {
            "address": "stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi",
            "role": 0,
          },
          {
            "address": "Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb",
            "role": 0,
          },
          {
            "address": "8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC",
            "role": 0,
          },
          {
            "address": "Hr9pzexrBge3vgmBNRR8u42CNQgBXdHm4UkUN2DH4a7r",
            "role": 0,
          },
          {
            "address": "2aMLkB5p5gVvCwKkdSo5eZAL1WwhZbxezQr1wxiynRhq",
            "role": 0,
          },
          {
            "address": "HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S",
            "role": 0,
          },
          {
            "address": "GVqitNXDVx1PdG47PMNeNEoHSEnVNqybW7E8NckmSJ2R",
            "role": 0,
          },
        ],
        "metadata": null,
        "oneReceiptTokenAsSOL": 0n,
        "receiptTokenDecimals": 9,
        "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
        "receiptTokenSupply": 0n,
        "supportedAssets": [
          {
            "decimals": 9,
            "depositable": true,
            "mint": null,
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1000000000n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": null,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": 1970-01-01T00:00:00.000Z,
            "withdrawalUserReservedAmount": 0n,
          },
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
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": 1970-01-01T00:00:00.000Z,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": "h7veGmqGWmFPe2vbsrKVNARvucfZ2WKCXUvJBmbJ86Q",
      }
    `);
  });

  test('restaking.fragSOL.fund.resolve', async () => {
    await expect(ctx.fund.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "assetStrategies": [
          {
            "solAccumulatedDepositAmount": 0n,
            "solAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "solDepositable": true,
            "solWithdrawable": true,
            "solWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "solWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 0n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": false,
            "tokenMint": "bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 0n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": false,
            "tokenMint": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 0n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": false,
            "tokenMint": "BNso1VUJnh4zcfpZa6986Ea66P6TCp59hvtNJ8b1X85",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 0n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": false,
            "tokenMint": "Bybit2vBJGhPF52GBdNaQfUJ6ZpThSgHBobjWZpLPb4B",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
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
        "restakingVaultStrategies": [
          {
            "compoundingRewardTokenMints": [
              "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            ],
            "delegations": [
              {
                "operator": "FzZ9EXmHv7ANCXijpALUBzCza6wYNprnsfaEHuoNx9sE",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 92n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "29rxXT5zbTR1ctiooHtb1Sa1TD4odzhQHsrLz3D78G5w",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 0n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "LKFpfXtBkH5b7D9mo8dPcjCLZCZpmLQC9ELkbkyVdah",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 92n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "GZxp4e2Tm3Pw9GyAaxuF6odT3XkRM96jpZkp3nxhoK4Y",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 2n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "CA8PaNSoFWzvbCJ2oK3QxBEutgyHSTT5omEptpj8YHPY",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 3n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "7yofWXChEHkPTSnyFdKx2Smq5iWVbGB4P1dkdC6zHWYR",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 1n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "BFEsrxFPsBcY2hR5kgyfKnpwgEc8wYQdngvRukLQXwG2",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 4n,
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
                "tokenAllocationWeight": 92n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "EkroMQiZJfphVd9iPvR4zMCHasTW72Uh1mFYkTxtQuY6",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 0n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "574DmorRvpaYrSrBRUwAjG7bBmrZYiTW3Fc8mvQatFqo",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 92n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "C6AF8qGCo2dL815ziRCmfdbFeL5xbRLuSTSZzTGBH68y",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 0n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "6AxtdRGAaiAyqcwxVBHsH3xtqCbQuffaiE4epT4koTxk",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 0n,
                "tokenRedelegatingAmount": 0n,
              },
            ],
            "distributingRewardTokenMints": [
              "REALSWTCH7J8JdafNBLZpfSCLiFwpMCqod2RpkU4RNn",
            ],
            "pricingSource": {
              "__kind": "JitoRestakingVault",
              "address": "HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S",
            },
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "vault": "HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S",
          },
        ],
        "tokenSwapStrategies": [],
      }
    `);
  });

  test(`restaking.fragSOL.reward.resolve`, async () => {
    await expectMasked(ctx.reward.resolve(true)).resolves
      .toMatchInlineSnapshot(`
      {
        "basePool": {
          "contribution": "MASKED(contribution)",
          "customContributionAccrualRateEnabled": false,
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
                "decimals": 9,
                "description": "Switchboard Token",
                "id": 1,
                "mint": "FSWSBMV5EB7J8JdafNBLZpfSCLiFwpMCqod2RpkU4RNn",
                "name": "SWTCH",
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
        "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
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
            "description": "Switchboard Token",
            "id": 1,
            "mint": "FSWSBMV5EB7J8JdafNBLZpfSCLiFwpMCqod2RpkU4RNn",
            "name": "SWTCH",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          },
        ],
      }
    `);
  });

  /** 2. deposit **/
  test('user can deposit SOL', async () => {
    await expectMasked(
      user1.deposit.execute(
        { assetMint: null, assetAmount: 5_000_000_000n },
        { signers: [signer1] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": 5000000000n,
          "assetMint": null,
          "metadata": null,
        },
        "events": {
          "unknown": [],
          "userCreatedOrUpdatedFundAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "userFundAccount": "47srXvirv37rsKhrVxtz7JWGq4CE2Ao4vjFUvTNdvg92",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "userRewardAccount": "9XZgibwtji6havXCPHKRoqpnR7MJUYgavQKCvDWALXGR",
          },
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 5000000000n,
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "mintedReceiptTokenAmount": 5000000000n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "supportedTokenMint": {
              "__option": "None",
            },
            "updatedUserRewardAccounts": [
              "9XZgibwtji6havXCPHKRoqpnR7MJUYgavQKCvDWALXGR",
            ],
            "user": "EhxcijcPKVdQ9zTSXGeLrgSEFJjbiNiC34j9prg3St29",
            "userFundAccount": "47srXvirv37rsKhrVxtz7JWGq4CE2Ao4vjFUvTNdvg92",
            "userReceiptTokenAccount": "BWfL432qksE6DpBEpRsuqaq4U6GdgPz1PGXKXNQkfr8M",
            "userSupportedTokenAccount": {
              "__option": "None",
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

    await expect(user1.resolve()).resolves.toMatchInlineSnapshot(`
      {
        "lamports": 94962596960n,
        "maxWithdrawalRequests": 4,
        "receiptTokenAmount": 5000000000n,
        "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
        "supportedAssets": [
          {
            "amount": 94962596960n,
            "decimals": 9,
            "depositable": true,
            "mint": null,
            "program": null,
            "withdrawable": true,
          },
          {
            "amount": 100000000000n,
            "decimals": 9,
            "depositable": true,
            "mint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
        ],
        "user": "EhxcijcPKVdQ9zTSXGeLrgSEFJjbiNiC34j9prg3St29",
        "withdrawalRequests": [],
        "wrappedTokenAmount": 0n,
        "wrappedTokenMint": "h7veGmqGWmFPe2vbsrKVNARvucfZ2WKCXUvJBmbJ86Q",
      }
    `);
  });

  test('user can deposit token with SPLStakePool pricing source', async () => {
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
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 5000000000n,
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "mintedReceiptTokenAmount": 5803579770n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            },
            "updatedUserRewardAccounts": [
              "9XZgibwtji6havXCPHKRoqpnR7MJUYgavQKCvDWALXGR",
            ],
            "user": "EhxcijcPKVdQ9zTSXGeLrgSEFJjbiNiC34j9prg3St29",
            "userFundAccount": "47srXvirv37rsKhrVxtz7JWGq4CE2Ao4vjFUvTNdvg92",
            "userReceiptTokenAccount": "BWfL432qksE6DpBEpRsuqaq4U6GdgPz1PGXKXNQkfr8M",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "4uGht5ZgiTn77KERTVtMm4WpxTeztWmpxgXhkNYBbXcQ",
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
    ).resolves.toBeGreaterThan(5000000000n * 2n);

    await expect(user1.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "lamports": 94962596960n,
        "maxWithdrawalRequests": 4,
        "receiptTokenAmount": 10803579770n,
        "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
        "supportedAssets": [
          {
            "amount": 94962596960n,
            "decimals": 9,
            "depositable": true,
            "mint": null,
            "program": null,
            "withdrawable": true,
          },
          {
            "amount": 95000000000n,
            "decimals": 9,
            "depositable": true,
            "mint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
        ],
        "user": "EhxcijcPKVdQ9zTSXGeLrgSEFJjbiNiC34j9prg3St29",
        "withdrawalRequests": [],
        "wrappedTokenAmount": 0n,
        "wrappedTokenMint": "h7veGmqGWmFPe2vbsrKVNARvucfZ2WKCXUvJBmbJ86Q",
      }
    `);

    await expect(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "G45gQa12Uwvnrp2Yb9oWTSwZSEHZWL71QDWvyLz23bNc",
        "__pricingSources": [
          {
            "address": "stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi",
            "role": 0,
          },
          {
            "address": "Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb",
            "role": 0,
          },
          {
            "address": "8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC",
            "role": 0,
          },
          {
            "address": "Hr9pzexrBge3vgmBNRR8u42CNQgBXdHm4UkUN2DH4a7r",
            "role": 0,
          },
          {
            "address": "2aMLkB5p5gVvCwKkdSo5eZAL1WwhZbxezQr1wxiynRhq",
            "role": 0,
          },
          {
            "address": "HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S",
            "role": 0,
          },
          {
            "address": "GVqitNXDVx1PdG47PMNeNEoHSEnVNqybW7E8NckmSJ2R",
            "role": 0,
          },
        ],
        "metadata": null,
        "oneReceiptTokenAsSOL": 1000000000n,
        "receiptTokenDecimals": 9,
        "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
        "receiptTokenSupply": 10803579770n,
        "supportedAssets": [
          {
            "decimals": 9,
            "depositable": true,
            "mint": null,
            "oneTokenAsReceiptToken": 1000000000n,
            "oneTokenAsSol": 1000000000n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 5000000000n,
            "operationTotalAmount": 5000000000n,
            "program": null,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 10803579770n,
            "withdrawalLastBatchProcessedAt": 1970-01-01T00:00:00.000Z,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "oneTokenAsReceiptToken": 1160715954n,
            "oneTokenAsSol": 1160715954n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 5000000000n,
            "operationTotalAmount": 5000000000n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 5803579770n,
            "withdrawalLastBatchProcessedAt": 1970-01-01T00:00:00.000Z,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": "h7veGmqGWmFPe2vbsrKVNARvucfZ2WKCXUvJBmbJ86Q",
      }
    `);
  });

  test('user can withdraw receipt tokens as SOL', async () => {
    let {
      receiptTokenSupply: expectedReceiptTokenSupply,
      oneReceiptTokenAsSOL,
    } = await ctx.resolve(true).then((data) => data!);

    for (let i = 1; i <= 4; i++) {
      const receiptTokenAmount = 23_456_789n * BigInt(i);
      expectedReceiptTokenSupply -= receiptTokenAmount;

      await expect(
        user1.requestWithdrawal.execute(
          {
            assetMint: null,
            receiptTokenAmount: receiptTokenAmount,
          },
          { signers: [signer1] }
        )
      ).resolves.toMatchObject({
        events: {
          userRequestedWithdrawalFromFund: {
            supportedTokenMint: { __option: 'None' },
            requestedReceiptTokenAmount: receiptTokenAmount,
          },
        },
      });
    }
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'EnqueueWithdrawalBatch',
    });
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'ProcessWithdrawalBatch',
    });

    for (let i = 1; i <= 4; i++) {
      const res = await user1.withdraw.execute(
        {
          assetMint: null,
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

    await expect(
      ctx.resolve(true),
      'receipt token supply reduced as withdrawal reqs being processed but the price maintains'
    ).resolves.toMatchObject({
      receiptTokenSupply: expectedReceiptTokenSupply,
      oneReceiptTokenAsSOL: oneReceiptTokenAsSOL,
    });
  });
});
