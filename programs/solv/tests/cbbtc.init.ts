import type { TestSuiteContext } from '../../testutil';

export function initializeCBBTCVault(testCtx: TestSuiteContext) {
  const { validator, solv, sdk, feePayer } = testCtx;

  const ctx = solv.cbBTC;
  const admin = '7hc9hViLRVvoTkJj8515EPM6CuJa7Sep8v5AFoTE8jYE'; // local admin
  const initializationTasks = [
    // initialize receipt token mint and vault
    () =>
      ctx.initialize.execute({
        admin: admin,
        receiptTokenMint: solv.knownAddresses.cbBTCVRT,
        supportedTokenMint: solv.knownAddresses.cbBTCVST,
      }),
  ].reduce(
    async (prevLogs, task) => {
      const logs = await prevLogs;
      const ctx = await task();
      if (ctx?.result?.meta?.logMessages) {
        logs.push(ctx.result.meta.logMessages);
      }
      return logs;
    },
    Promise.resolve([] as (readonly string[])[])
  );

  return { ...testCtx, initializationTasks, admin };
}
