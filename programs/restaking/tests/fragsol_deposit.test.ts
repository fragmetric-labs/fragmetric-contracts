import { afterAll, beforeAll, describe, expect, test } from 'vitest';
import { initializeFragSOL } from './fragsol';
import { createTestSuiteContext, expectMasked } from './utils';

describe('restaking.fragSOL deposit test', async () => {
  const testCtx = await createTestSuiteContext();
  const { validator, restaking, initializationTasks } = initializeFragSOL(testCtx);

  beforeAll(async () => {
    await initializationTasks;
    await Promise.all(
      ['Daniel', 'Tommy', 'Ryn', 'Terry'].map((seed) =>
        validator
          .newSigner(seed, 100_000_000_000n)
          .then((signer) =>
            Promise.all([
              validator.airdropToken(
                signer.address,
                'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
                100_000_000_000n
              ),
            ])
          )
      )
    );
  });
  afterAll(() => validator.quit());

  const ctx = restaking.fragSOL;

  test('user can deposit SOL', async () => {
    const signer1 = await validator.getSigner('Daniel');
    const user1 = ctx.user(signer1);
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
            "userFundAccount": "Fv4kxFDM9b56gyHkSv3mH7vK7Huye2APGn16vLzsrDxo",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "userRewardAccount": "4wpdD9G3nKARC75ZQaQx1w5hd7yXLs8ydzza31nr9Eib",
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
              "4wpdD9G3nKARC75ZQaQx1w5hd7yXLs8ydzza31nr9Eib",
            ],
            "user": "JDdS2vKWBaT13BpFLFhDgDkt9aQKgB98m7wmtyKp9UA1",
            "userFundAccount": "Fv4kxFDM9b56gyHkSv3mH7vK7Huye2APGn16vLzsrDxo",
            "userReceiptTokenAccount": "6epF3pjCcqUPpqoo7HaxLayh152RpJ9zu3ZqKdFKh9qG",
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
        "user": "JDdS2vKWBaT13BpFLFhDgDkt9aQKgB98m7wmtyKp9UA1",
        "withdrawalRequests": [],
        "wrappedTokenAmount": 0n,
        "wrappedTokenMint": "h7veGmqGWmFPe2vbsrKVNARvucfZ2WKCXUvJBmbJ86Q",
      }
    `);
  });

  test('user can deposit supported tokens', async () => {
    const signer1 = await validator.getSigner('Daniel');
    const user1 = ctx.user(signer1);
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
              "4wpdD9G3nKARC75ZQaQx1w5hd7yXLs8ydzza31nr9Eib",
            ],
            "user": "JDdS2vKWBaT13BpFLFhDgDkt9aQKgB98m7wmtyKp9UA1",
            "userFundAccount": "Fv4kxFDM9b56gyHkSv3mH7vK7Huye2APGn16vLzsrDxo",
            "userReceiptTokenAccount": "6epF3pjCcqUPpqoo7HaxLayh152RpJ9zu3ZqKdFKh9qG",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "GCdYC75hGuraUcrQpAxgnLwwAyaEktDGXRvsjzMLkYME",
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
        "user": "JDdS2vKWBaT13BpFLFhDgDkt9aQKgB98m7wmtyKp9UA1",
        "withdrawalRequests": [],
        "wrappedTokenAmount": 0n,
        "wrappedTokenMint": "h7veGmqGWmFPe2vbsrKVNARvucfZ2WKCXUvJBmbJ86Q",
      }
    `);

    await expect(ctx.resolve(true)).resolves.toMatchInlineSnapshot();
  });
});
