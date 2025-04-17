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
        ],
        "metadata": null,
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
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
            "withdrawalLastBatchProcessedAt": 1970-01-01T00:00:00.000Z,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "oneTokenAsReceiptToken": 100000000n,
            "oneTokenAsSol": 647695086539n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
            "withdrawalLastBatchProcessedAt": 1970-01-01T00:00:00.000Z,
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
        ],
        "metadata": null,
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
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
            "withdrawalLastBatchProcessedAt": 1970-01-01T00:00:00.000Z,
          },
          {
            "decimals": 8,
            "depositable": true,
            "mint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
            "oneTokenAsReceiptToken": 100000000n,
            "oneTokenAsSol": 647695086539n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": true,
            "withdrawalLastBatchProcessedAt": 1970-01-01T00:00:00.000Z,
          },
        ],
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);
  });
};
