import { expect, test } from 'vitest';
import { expectMasked } from '../../testutil';
import { initializeZBTCVault } from './zbtc';

export const zBTCVaultConfigurationTest = async (
  testCtx: ReturnType<typeof initializeZBTCVault>
) => {
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
};
