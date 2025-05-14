import { describe, expect, test } from 'vitest';
import { RestakingProgram } from './program';
import { testSignerResolver } from './testing_fixture';

describe('RestakingRewardOperatorContext on devnet', async () => {
  const keypair = await testSignerResolver();
  const program = RestakingProgram.devnet(process.env.SOLANA_RPC_DEVNET);

  test('can execute updatePoolsTransaction', async () => {
    await expect(
      program.fragSOL.reward.updatePools.execute(null, {
        feePayer: keypair,
      })
    ).resolves.toMatchObject({
      events: {
        unknown: [],
        operatorUpdatedRewardPools: {
          receiptTokenMint: 'FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo',
          rewardAccount: 'EKytoaHHhKeQ5B4ewWq7LjyTY2to1wLPJqpVRkRA8HQk',
        },
      },
      succeeded: true,
    });
  });
});
