import { expect, test } from 'vitest';
import { initializeFragSOL } from './fragsol';

export const fragSOLPlaygroundTest = async (
  testCtx: ReturnType<typeof initializeFragSOL>
) => {
  const { validator, feePayer, restaking, initializationTasks } = testCtx;
  const ctx = restaking.fragSOL;

  const signer1 = await validator
    .newSigner('fragSOLPlaygroundTestSigner1', 100_000_000_000n)
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

  test(`new test example`, async () => {
    // implement some test suite then merge into an existing suite to reduce number of test suites if possible
    expect(true).toBeTruthy();
  });
};
