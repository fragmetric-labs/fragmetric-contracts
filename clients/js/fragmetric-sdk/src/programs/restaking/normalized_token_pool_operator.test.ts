import { describe, expect, test } from 'vitest';
import { RestakingProgram } from './program';
import { testSignerResolver } from './testing_fixture';

describe('RestakingNormalizedTokenPoolOperatorContext on devnet', async () => {
  const keypair = await testSignerResolver();
  const program = RestakingProgram.devnet();

  test('can execute updatePricesTransaction', async () => {
    await expect(
      program.fragSOL.normalizedTokenPool.updatePrices.execute(null, {
        feePayer: keypair,
      })
    ).resolves.toMatchObject({
      events: {
        unknown: [],
        operatorUpdatedNormalizedTokenPoolPrices: {
          normalizedTokenMint: 'nSoLnkrvh2aY792pgCNT6hzx84vYtkviRzxvhf3ws8e',
          normalizedTokenPoolAccount:
            '36znkkBhTNJY6PzidFN7vwuZysbn8P8hz4LBk3AZn33Z',
        },
      },
      succeeded: true,
    });
  });
});
