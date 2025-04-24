import type { restakingTypes } from '@fragmetric-labs/sdk';
import { isSome } from '@solana/kit';
import { expect, test } from 'vitest';
import { initializeFragBTC } from './fragbtc';
import { expectMasked } from './utils';

export const fragBTCDepositTest = async (
  testCtx: ReturnType<typeof initializeFragBTC>
) => {
  const { validator, feePayer, restaking, initializationTasks } = testCtx;
  const ctx = restaking.fragBTC;

  const signer1 = await validator
    .newSigner('fragBTCDepositTestSigner1', 100_000_000_000n)
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
    });
  const user1 = ctx.user(signer1);

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
            "userFundAccount": "FP3ZDLWnKWcgEtDX8Hv89pk9nDG4DexMqjtQZAEL9SCU",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "userRewardAccount": "3WyAUizuFyR6U2mkJeRPKfkC8nwcWjo8dEKUBRFwx8XN",
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
              "3WyAUizuFyR6U2mkJeRPKfkC8nwcWjo8dEKUBRFwx8XN",
            ],
            "user": "J6n5QRwPqcZMFK3AK8bgQhmpnHKzLteziJEAmkEnzG1z",
            "userFundAccount": "FP3ZDLWnKWcgEtDX8Hv89pk9nDG4DexMqjtQZAEL9SCU",
            "userReceiptTokenAccount": "3jdG7MWQbVTCANTouAhS5N6RxYABg3H2whnEQYLrHBiL",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "4beyqmiDcGH7YbKegj71Bow8wtzM9p7bnKtFTBBPZ6kj",
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
        "user": "J6n5QRwPqcZMFK3AK8bgQhmpnHKzLteziJEAmkEnzG1z",
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
              "3WyAUizuFyR6U2mkJeRPKfkC8nwcWjo8dEKUBRFwx8XN",
            ],
            "user": "J6n5QRwPqcZMFK3AK8bgQhmpnHKzLteziJEAmkEnzG1z",
            "userFundAccount": "FP3ZDLWnKWcgEtDX8Hv89pk9nDG4DexMqjtQZAEL9SCU",
            "userReceiptTokenAccount": "3jdG7MWQbVTCANTouAhS5N6RxYABg3H2whnEQYLrHBiL",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "AAKuDvc2NNWqWpkB3dFhcsGfSoS4kSeVe3kPKQ3UELyp",
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
        "user": "J6n5QRwPqcZMFK3AK8bgQhmpnHKzLteziJEAmkEnzG1z",
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
        "receiptTokenSupply": 200000000n,
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

  test('funds supporting only a single token and tokens pegged to it must issue receipt tokens at a 1:1 ratio until additional yield is compounded', async () => {
    let expectedReceiptTokenSupply = await ctx
      .resolve(true)
      .then((data) => data!.receiptTokenSupply);

    for (let i = 1; i <= 9; i++) {
      const assetMint = ['zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg', 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij', '3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh'][i % 3];
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
      const assetMint = ['zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg', 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij'][i % 2];
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
};
