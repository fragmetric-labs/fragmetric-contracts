import { afterAll, beforeAll, describe, expect, test } from 'vitest';
import { createTestSuiteContext, expectMasked } from '../../testutil';
import { initializeZBTCVault } from './zbtc.init';

describe('solv.zBTC vault test', async () => {
  const testCtx = initializeZBTCVault(
    await createTestSuiteContext({ programs: { restaking: false } })
  );

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  /** 1. configuration **/
  const { validator, feePayer, solv, initializationTasks } = testCtx;
  const ctx = solv.zBTC;

  test(`solv.zBTC initializationTasks snapshot`, async () => {
    await expectMasked(initializationTasks).resolves.toMatchSnapshot();
  });

  test(`solv.zBTC.resolve`, async () => {
    await expect(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "receiptTokenMint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
        "supportedTokenMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
        "vault": "H6pGcL98Rkz2aV8pq5jDEMdtrnogAmhUM5w8RAsddeB6",
      }
    `);
  });
});
