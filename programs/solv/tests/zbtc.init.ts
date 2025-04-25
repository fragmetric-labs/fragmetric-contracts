import type { TestSuiteContext } from '../../testutil';

export function initializeZBTCVault(testCtx: TestSuiteContext) {
  const { validator, solv, restaking, sdk, feePayer } = testCtx;
  const { MAX_U64 } = sdk;

  const ctx = solv.zBTC;
  const initializationTasks = [
    // initialize receipt token mint and transfer hook metadata
    () =>
      ctx.initializeMint.execute({
        authority: feePayer, // should be restaking fund account for integration
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

  return { ...testCtx, initializationTasks };
}
