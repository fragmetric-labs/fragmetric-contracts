import { afterAll, beforeAll, describe, test } from 'vitest';
import { createTestSuiteContext, expectMasked } from '../../testutil';
import { initializeFragUSD } from './fragusd.init';

describe('restaking.fragSWTCH test', async () => {
  const testCtx = initializeFragUSD(await createTestSuiteContext());

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  const { validator, feePayer, restaking, initializationTasks, sdk } = testCtx;
  const ctx = restaking.fragUSD;

  const [signer1, signer2] = await Promise.all([
    validator
      .newSigner('fragUSDDepositTestSigner1', 100_000_000_000n)
      .then(async (signer) => {
        await Promise.all([
          validator.airdropToken(
            signer.address,
            'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v',
            100_000_000_000n
          ),
        ]);
        return signer;
      }),
    validator
      .newSigner('fragUSDDepositTestSigner2', 100_000_000_000n)
      .then(async (signer) => {
        await Promise.all([
          validator.airdropToken(
            signer.address,
            'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v',
            100_000_000_000n
          ),
        ]);
        return signer;
      }),
    validator.airdrop(restaking.knownAddresses.fundManager, 100_000_000_000n),
  ]);
  const user1 = ctx.user(signer1);
  const user2 = ctx.user(signer2);

  test('restaking.fragSWTCH initializationTasks snapshot', async () => {
    await expectMasked(initializationTasks).resolves.toMatchSnapshot();
  });

  test('restaking.fragSWTCH.resolve', async () => {
    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot();
  });
});
