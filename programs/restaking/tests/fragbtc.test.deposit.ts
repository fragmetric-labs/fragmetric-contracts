import { beforeAll, expect, test } from 'vitest';
import { initializeFragBTC } from './fragbtc';
import { expectMasked } from './utils';

export const fragBTCDepositTest = async (
  testCtx: ReturnType<typeof initializeFragBTC>
) => {
  const { validator, feePayer, restaking, initializationTasks } = testCtx;
  const ctx = restaking.fragBTC;

  beforeAll(async () => {
    await Promise.all(
      ['Daniel', 'Tommy', 'Ryn', 'Terry'].map((seed) =>
        validator
          .newSigner(seed, 100_000_000_000n)
          .then((signer) =>
            Promise.all([
              validator.airdropToken(
                signer.address,
                'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
                100_000_000_000n
              ),
            ])
          )
      )
    );
  });

  test('user can deposit supported tokens', async () => {
    const signer1 = await validator.getSigner('Daniel');
    const user1 = ctx.user(signer1);
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
            "userFundAccount": "Feir6KevfNio6QeJMCPvdZ8G8ZWmAyen9LAkEsWtWMqW",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "userRewardAccount": "FGW5iBvit94zRUbj9X24gZkJmhhFzE5yNrsM5WafEXmc",
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
              "FGW5iBvit94zRUbj9X24gZkJmhhFzE5yNrsM5WafEXmc",
            ],
            "user": "JDdS2vKWBaT13BpFLFhDgDkt9aQKgB98m7wmtyKp9UA1",
            "userFundAccount": "Feir6KevfNio6QeJMCPvdZ8G8ZWmAyen9LAkEsWtWMqW",
            "userReceiptTokenAccount": "4Fxr8VKy9nUFjzbG8F6asjBNiC65Z4yDF1Cs1Y5DMDEy",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "4P6iSFUgRKwsc3W5U7hZGATPnnDo25Vbr2qnpMRuCvbd",
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
        ],
        "user": "JDdS2vKWBaT13BpFLFhDgDkt9aQKgB98m7wmtyKp9UA1",
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
        ],
        "wrappedTokenMint": "9mCDpsmPeozJKhdpYvNTJxi9i2Eaav3iwT5gzjN7VbaN",
      }
    `);
  });
};
