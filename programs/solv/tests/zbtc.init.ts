import type { TestSuiteContext } from '../../testutil';

export function initializeZBTCVault(testCtx: TestSuiteContext) {
  const { validator, solv, sdk, feePayer } = testCtx;

  // for test
  const knownAddresses = {
    vaultManager: '9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL', // local restaking - admin
    rewardManager: '5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx', // local restaking - fund manager
    fundManager: '4pfiCiaZsTf8aNKexuGYzhQLbg6iJKoYospSUmfTpL2K', // local solv - fund manager
    solvManager: 'BBiQ99GVfamTcqcYwLgji4k5giL3C8epmR3do1thYigw', // local solv - solv manager
    solvProtocolWallet: '4xqLe1ALAh8sbi2N2uEM5JXbhhVNKMVRg3L1m1E2Hfbv', // local solv - solv protocol wallet
  };

  const ctx = solv.zBTC;
  const initializationTasks = [
    // initialize receipt token mint and vault
    () => ctx.initializeReceiptTokenMint.execute(null),
    () => ctx.initializeOrUpdateAccount.execute(null),
    () =>
      ctx.setAdminRoles.execute({
        vaultManager: knownAddresses.vaultManager,
        rewardManager: knownAddresses.rewardManager,
        fundManager: knownAddresses.fundManager,
        solvManager: knownAddresses.solvManager,
      }),
    () =>
      ctx.delegateRewardTokenAccount.execute({
        mint: 'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
        delegate: knownAddresses.rewardManager,
      }),
    () =>
      ctx.setSolvProtocolWallet.execute({
        address: knownAddresses.solvProtocolWallet,
      }),
    () =>
      ctx.setSolvProtocolWithdrawalFeeRate.execute({
        feeRateBps: 80, // 0.8%
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

  return { ...testCtx, initializationTasks, knownAddresses };
}
