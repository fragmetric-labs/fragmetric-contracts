import { afterAll, beforeAll, describe, expect, test } from 'vitest';
import { createTestSuiteContext, expectMasked } from '../../testutil';
import { initializeCBBTCVault } from './cbbtc.init';

describe('solv.cbBTC test', async () => {
  const testCtx = initializeCBBTCVault(await createTestSuiteContext());

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  /** 1. configuration **/
  const { validator, feePayer, solv, initializationTasks, admin } = testCtx;
  const ctx = solv.cbBTC;

  test(`solv.cbBTC initializationTasks snapshot`, async () => {
    await expectMasked(initializationTasks).resolves.toMatchSnapshot();
  });

  test(`solv.cbBTC.resolve`, async () => {
    await expect(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "admin": "7hc9hViLRVvoTkJj8515EPM6CuJa7Sep8v5AFoTE8jYE",
        "delegateRewardTokenAdmin": "7hc9hViLRVvoTkJj8515EPM6CuJa7Sep8v5AFoTE8jYE",
        "delegatedRewardTokens": [],
        "receiptTokenDecimals": 8,
        "receiptTokenLockedAmount": 0n,
        "receiptTokenMint": "BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 0n,
        "supportedTokenAmount": 0n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij",
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
      }
    `);
  });
});
