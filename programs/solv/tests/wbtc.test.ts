import { afterAll, beforeAll, describe, expect, test } from 'vitest';
import { createTestSuiteContext, expectMasked } from '../../testutil';
import { initializeWBTCVault } from './wbtc.init';

describe('solv.wBTC test', async () => {
  const testCtx = initializeWBTCVault(await createTestSuiteContext());

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  /** 1. configuration **/
  const { validator, feePayer, solv, initializationTasks, admin } = testCtx;
  const ctx = solv.wBTC;

  test(`solv.wBTC initializationTasks snapshot`, async () => {
    await expectMasked(initializationTasks).resolves.toMatchSnapshot();
  });

  test(`solv.wBTC.resolve`, async () => {
    await expect(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "admin": "7hc9hViLRVvoTkJj8515EPM6CuJa7Sep8v5AFoTE8jYE",
        "delegateRewardTokenAdmin": "7hc9hViLRVvoTkJj8515EPM6CuJa7Sep8v5AFoTE8jYE",
        "delegatedRewardTokens": [],
        "receiptTokenDecimals": 8,
        "receiptTokenLockedAmount": 0n,
        "receiptTokenMint": "4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 0n,
        "supportedTokenAmount": 0n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
      }
    `);
  });
});
