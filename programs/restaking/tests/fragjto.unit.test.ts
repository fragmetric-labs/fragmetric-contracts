import { createKeyPairSignerFromBytes } from '@solana/kit';
import { afterAll, beforeAll, describe, expect, test } from 'vitest';
import { createTestSuiteContext } from '../../testutil';
import { initializeFragJTO } from './fragjto.unit.init';

describe('restaking.fragJTO unit test', async () => {
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
  await Promise.all([
    validator.airdrop(restaking.knownAddresses.fundManager, 100_000_000_000n),
  ]);

  test('remove token swap strategy', async () => {
    const fund_1 = await ctx.fund.resolve(true);
    expect(fund_1.tokenSwapStrategies).toHaveLength(1);

    await expect(
      ctx.fund.removeTokenSwapStrategy.execute({
        fromTokenMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
        toTokenMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
        swapSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: 'G2FiE1yn9N9ZJx5e1E2LxxMnHvb1H3hCuHLPfKJ98smA',
        },
      })
    ).resolves.not.toThrow();

    await expect(
      ctx.fund.removeTokenSwapStrategy.execute({
        fromTokenMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL', // invalid from token mint
        toTokenMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
        swapSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: 'G2FiE1yn9N9ZJx5e1E2LxxMnHvb1H3hCuHLPfKJ98smA',
        },
      })
    ).rejects.toThrowError('Transaction simulation failed'); // fund: token swap strategy not found.
    await expect(
      ctx.fund.removeTokenSwapStrategy.execute({
        fromTokenMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
        toTokenMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn', // invalid to token mint
        swapSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: 'G2FiE1yn9N9ZJx5e1E2LxxMnHvb1H3hCuHLPfKJ98smA',
        },
      })
    ).rejects.toThrowError('Transaction simulation failed'); // fund: token swap strategy not found.
    await expect(
      ctx.fund.removeTokenSwapStrategy.execute({
        fromTokenMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
        toTokenMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
        swapSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL', // invalid swap source
        },
      })
    ).rejects.toThrowError('Transaction simulation failed'); // fund: token swap strategy validation error.

    const fund_2 = await ctx.fund.resolve(true);
    expect(fund_2.tokenSwapStrategies).toHaveLength(0);
  });
});
