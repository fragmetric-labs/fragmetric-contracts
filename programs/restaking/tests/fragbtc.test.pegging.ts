import { expect, test } from 'vitest';
import { initializeFragBTC } from './fragbtc';

export const fragBTCPeggingTest = async (
  testCtx: ReturnType<typeof initializeFragBTC>
) => {
  const { validator, feePayer, restaking, initializationTasks } = testCtx;
  const ctx = restaking.fragBTC;

  const signer1 = await validator
    .newSigner('fragBTCPeggingTestSigner1', 100_000_000_000n)
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

  test('funds supporting only a single token and tokens pegged to it must issue receipt tokens at a 1:1 ratio until additional yield is compounded', async () => {
    let expectedReceiptTokenSupply = await ctx
      .resolve(true)
      .then((data) => data!.receiptTokenSupply);

    for (let i = 1; i <= 10; i++) {
      const assetMint =
        i % 2 == 0
          ? 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg'
          : 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij';
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

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      receiptTokenSupply: expectedReceiptTokenSupply,
    });
  });

  test('funds only with pegged tokens maintain pegging even with repetitive withdrawals', async () => {
    let expectedReceiptTokenSupply = await ctx
      .resolve(true)
      .then((data) => data!.receiptTokenSupply);

    for (let i = 1; i <= 4; i++) {
      const receiptTokenAmount = 23_456_789n * BigInt(i);
      const assetMint =
        i % 2 == 0
          ? 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg'
          : 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij';
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
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'ProcessWithdrawalBatch',
    });

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      receiptTokenSupply: expectedReceiptTokenSupply,
    });
  });
};
