import { expect, test } from 'vitest';
import { expectMasked } from '../../testutil';
import { initializeFragSOL } from './fragsol';

export const fragSOLDepositTest = async (
  testCtx: ReturnType<typeof initializeFragSOL>
) => {
  const { validator, feePayer, restaking, initializationTasks } = testCtx;
  const ctx = restaking.fragSOL;

  const signer1 = await validator
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
    });
  const user1 = ctx.user(signer1);

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
};
