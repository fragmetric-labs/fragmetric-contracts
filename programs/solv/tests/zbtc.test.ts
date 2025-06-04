import { afterAll, beforeAll, describe, expect, test } from 'vitest';
import { createTestSuiteContext, expectMasked } from '../../testutil';
import { initializeZBTCVault } from './zbtc.init';

describe('solv.zBTC test', async () => {
  const testCtx = initializeZBTCVault(await createTestSuiteContext());

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  /** 1. configuration **/
  const { validator, feePayer, solv, initializationTasks, knownAddresses } =
    testCtx;
  const ctx = solv.zBTC;

  await Promise.all([
    validator.airdropToken(
      knownAddresses.fundManager,
      'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
      100_0000_0000n
    ),
    validator.airdropToken(
      knownAddresses.solvProtocolWallet,
      'SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8',
      100_0000_0000n
    ),
    validator.airdropToken(
      knownAddresses.rewardManager,
      'ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq',
      100_000_000_000n
    ),
  ]);

  test(`solv.zBTC initializationTasks snapshot`, async () => {
    await expectMasked(initializationTasks).resolves.toMatchSnapshot();
  });

  test(`solv.zBTC.resolve`, async () => {
    await expect(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "4pfiCiaZsTf8aNKexuGYzhQLbg6iJKoYospSUmfTpL2K",
          "rewardManager": "5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx",
          "solvManager": "BBiQ99GVfamTcqcYwLgji4k5giL3C8epmR3do1thYigw",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [
          {
            "amount": 0n,
            "delegate": {
              "__option": "Some",
              "value": "5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx",
            },
            "mint": "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
          },
        ],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100000000n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 100000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 0n,
        "solvProtocolWallet": "4xqLe1ALAh8sbi2N2uEM5JXbhhVNKMVRg3L1m1E2Hfbv",
        "solvProtocolWithdrawalFeeRate": 0.008,
        "solvReceiptTokenAmount": 0n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 0n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
        "supportedTokenOperationReservedAmount": 0n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 0n,
            "requests": [],
            "supportedTokenDeductedFeeAmount": 0n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 0n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 0n,
            "requests": [],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 0n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 0n,
            "requests": [],
            "supportedTokenReceivableAmount": 0n,
          },
        },
      }
    `);
  });

  /** 1. deposit **/
  test(`deposit process for phase1`, async () => {
    // fund manager deposits
    await ctx.deposit.execute({
      payer: knownAddresses.fundManager,
      supportedTokenAmount: 1_0000_0000n,
    });

    await ctx.deposit.execute({
      payer: knownAddresses.fundManager,
      supportedTokenAmount: 1_2345_6789n,
    });

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsSupportedTokenAmount: 1_0000_0000n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 1_0000_0000n,
      receiptTokenSupply: 223456789n,
      supportedTokenAmount: 223456789n,
      supportedTokenOperationReservedAmount: 223456789n,
      solvReceiptTokenAmount: 0n,
      solvReceiptTokenOperationReservedAmount: 0n,
      solvReceiptTokenOperationReceivableAmount: 0n,
    });

    // solv manager confirms two deposits
    await ctx.confirmDeposits.execute(null);

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsSupportedTokenAmount: 1_0000_0000n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 1_0000_0000n,
      receiptTokenSupply: 223456789n,
      supportedTokenAmount: 0n,
      supportedTokenOperationReservedAmount: 0n,
      solvReceiptTokenAmount: 0n,
      solvReceiptTokenOperationReservedAmount: 0n,
      solvReceiptTokenOperationReceivableAmount: 223456789n,
    });

    await expect(
      ctx.solvProtocolWallet.supportedToken.resolve(true).then((a) => a.amount)
    ).resolves.toEqual(223456789n);

    // transfer srt to the vault with exact 1:1 rate + 1n
    await ctx.donate.execute({
      payer: knownAddresses.solvProtocolWallet,
      supportedTokenAmount: 0n,
      receiptTokenAmount: 0n,
      solvReceiptTokenAmount: 223456789n + 1n,
    });

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsSupportedTokenAmount: 1_0000_0000n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 1_0000_0000n,
      receiptTokenSupply: 223456789n,
      supportedTokenAmount: 0n,
      supportedTokenOperationReservedAmount: 0n,
      solvReceiptTokenAmount: 223456789n + 1n,
      solvReceiptTokenOperationReservedAmount: 0n,
      solvReceiptTokenOperationReceivableAmount: 223456789n,
    });

    // solv manager cannot complete with less value of srt
    await expect(
      ctx.completeDeposits.execute({
        redeemedSolvReceiptTokenAmount: 2_2345_6789n + 1n,
        newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_000_0000_000000n,
      })
    ).rejects.toThrow();

    // solv manager cannot complete with too rapid srt price incrase
    await expect(
      ctx.completeDeposits.execute({
        redeemedSolvReceiptTokenAmount: (2_2345_6789n * 10n) / 13n,
        newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_3000_0000_000000n,
      })
    ).rejects.toThrow();

    // solv manager can complete with same redemption rate
    await expect(
      ctx.completeDeposits.execute({
        redeemedSolvReceiptTokenAmount: 2_2345_6789n,
        newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_0000_0000_000000n,
      })
    ).resolves.not.toThrow();

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsSupportedTokenAmount: 100000000n,
      oneReceiptTokenAsMicroSupportedTokenAmount: 100000000000000n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 100000000n,
      oneSolvReceiptTokenAsMicroSupportedTokenAmount: 100000000000000n,
      receiptTokenSupply: 223456789n,
      supportedTokenAmount: 0n,
      supportedTokenOperationReservedAmount: 0n,
      solvReceiptTokenAmount: 223456789n + 1n,
      solvReceiptTokenOperationReservedAmount: 223456789n,
      solvReceiptTokenOperationReceivableAmount: 0n,
    });

    // once again
    await ctx.deposit.execute({
      payer: knownAddresses.fundManager,
      supportedTokenAmount: 76_543_211n,
    });

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsSupportedTokenAmount: 100000000n,
      oneReceiptTokenAsMicroSupportedTokenAmount: 100000000000000n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 100000000n,
      oneSolvReceiptTokenAsMicroSupportedTokenAmount: 100000000000000n,
      receiptTokenSupply: 300000000n,
      supportedTokenAmount: 76_543_211n,
      supportedTokenOperationReservedAmount: 76_543_211n,
      solvReceiptTokenAmount: 223456789n + 1n,
      solvReceiptTokenOperationReservedAmount: 223456789n,
      solvReceiptTokenOperationReceivableAmount: 0n,
    });

    await ctx.confirmDeposits.execute(null);

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsSupportedTokenAmount: 100000000n,
      oneReceiptTokenAsMicroSupportedTokenAmount: 100000000000000n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 100000000n,
      oneSolvReceiptTokenAsMicroSupportedTokenAmount: 100000000000000n,
      receiptTokenSupply: 300000000n,
      supportedTokenAmount: 0n,
      supportedTokenOperationReservedAmount: 0n,
      solvReceiptTokenAmount: 223456789n + 1n,
      solvReceiptTokenOperationReservedAmount: 223456789n,
      solvReceiptTokenOperationReceivableAmount: 76_543_211n,
    });

    // transfer srt to the vault which is enough to meet exact 1:1.1 rate
    await ctx.donate.execute({
      payer: knownAddresses.solvProtocolWallet,
      supportedTokenAmount: 0n,
      receiptTokenAmount: 0n,
      solvReceiptTokenAmount: 69_584_737n, // 76_543_211n / 1.1
    });

    // too less amount with 1:1.05 rate
    await expect(
      ctx.completeDeposits.execute({
        redeemedSolvReceiptTokenAmount: 69_584_737n,
        newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_0500_0000_000000n,
      })
    ).rejects.toThrow();

    // proper amount with near 1:1.1 rate
    await expect(
      ctx.completeDeposits.execute({
        redeemedSolvReceiptTokenAmount: 69_584_737n,
        newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_0999_9999_123456n,
      })
    ).resolves.not.toThrow();

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsMicroSupportedTokenAmount: 107448558666666n,
      oneReceiptTokenAsSupportedTokenAmount: 107448558n,
      oneSolvReceiptTokenAsMicroSupportedTokenAmount: 109999999123456n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 109999999n,
      receiptTokenSupply: 300000000n,
      supportedTokenAmount: 0n,
      supportedTokenOperationReservedAmount: 0n,
      solvReceiptTokenAmount: 293041527n,
      solvReceiptTokenOperationReceivableAmount: 0n,
      solvReceiptTokenOperationReservedAmount: 293041526n,
    });

    const vaultReceiptTokenValueNumerator = 3_0000_0000n * 1_0744_8558_666666n;
    const vaultNetValueNumerator = 2_9304_1526n * 1_0999_9999_123456n;
    expect(
      (vaultReceiptTokenValueNumerator - vaultNetValueNumerator) /
        1_0000_0000_000000n
    ).toEqual(0n);
  });

  /** 2. withdrawal **/
  test(`withdrawal process for phase1`, async () => {
    // fund manager request withdrawals
    await ctx.requestWithdrawal.execute({
      payer: knownAddresses.fundManager,
      receiptTokenAmount: 1_0000_0000n,
    });

    await ctx.requestWithdrawal.execute({
      payer: knownAddresses.fundManager,
      receiptTokenAmount: 2_0000_0000n,
    });

    await expect(ctx.fundManager.receiptToken.resolve(true)).resolves
      .toMatchInlineSnapshot(`
      {
        "amount": 0n,
        "closeAuthority": {
          "__option": "None",
        },
        "delegate": {
          "__option": "None",
        },
        "delegatedAmount": 0n,
        "isNative": {
          "__option": "None",
        },
        "mint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
        "owner": "4pfiCiaZsTf8aNKexuGYzhQLbg6iJKoYospSUmfTpL2K",
        "state": 1,
      }
    `);

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      receiptTokenSupply: 0n,
      solvReceiptTokenAmount: 293041527n,
      solvReceiptTokenOperationReceivableAmount: 0n,
      solvReceiptTokenOperationReservedAmount: 0n,
      supportedTokenAmount: 0n,
      withdrawal: {
        enqueued: {
          receiptTokenEnqueuedAmount: 300000000n,
          requests: [
            {
              id: 1n,
              receiptTokenEnqueuedAmount: 100000000n,
              solvReceiptTokenLockedAmount: 97680509n,
              supportedTokenLockedAmount: 0n,
              supportedTokenTotalEstimatedAmount: 107448559n,
            },
            {
              id: 2n,
              receiptTokenEnqueuedAmount: 200000000n,
              solvReceiptTokenLockedAmount: 195361017n,
              supportedTokenLockedAmount: 0n,
              supportedTokenTotalEstimatedAmount: 214897116n,
            },
          ],
          solvReceiptTokenLockedAmount: 293041526n,
          supportedTokenLockedAmount: 0n,
        },
        processing: {
          receiptTokenProcessingAmount: 0n,
          requests: [],
          supportedTokenReceivableAmount: 0n,
        },
        completed: {
          receiptTokenProcessedAmount: 0n,
          requests: [],
          supportedTokenDeductedFeeAmount: 0n,
          supportedTokenExtraClaimableAmount: 0n,
          supportedTokenTotalClaimableAmount: 0n,
        },
      },
    });

    // solv manager confirm requests
    await ctx.confirmWithdrawalRequests.execute(null);
    await expect(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "4pfiCiaZsTf8aNKexuGYzhQLbg6iJKoYospSUmfTpL2K",
          "rewardManager": "5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx",
          "solvManager": "BBiQ99GVfamTcqcYwLgji4k5giL3C8epmR3do1thYigw",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [
          {
            "amount": 0n,
            "delegate": {
              "__option": "Some",
              "value": "5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx",
            },
            "mint": "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
          },
        ],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 107448558666666n,
        "oneReceiptTokenAsSupportedTokenAmount": 107448558n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 109999999123456n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 109999999n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 0n,
        "solvProtocolWallet": "4xqLe1ALAh8sbi2N2uEM5JXbhhVNKMVRg3L1m1E2Hfbv",
        "solvProtocolWithdrawalFeeRate": 0.008,
        "solvReceiptTokenAmount": 1n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 0n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
        "supportedTokenOperationReservedAmount": 0n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 0n,
            "requests": [],
            "supportedTokenDeductedFeeAmount": 0n,
            "supportedTokenExtraClaimableAmount": 0n,
            "supportedTokenTotalClaimableAmount": 0n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 0n,
            "requests": [],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 0n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 300000000n,
            "requests": [
              {
                "id": 1n,
                "receiptTokenEnqueuedAmount": 100000000n,
                "solvReceiptTokenLockedAmount": 97680509n,
                "supportedTokenLockedAmount": 0n,
                "supportedTokenTotalEstimatedAmount": 107448559n,
              },
              {
                "id": 2n,
                "receiptTokenEnqueuedAmount": 200000000n,
                "solvReceiptTokenLockedAmount": 195361017n,
                "supportedTokenLockedAmount": 0n,
                "supportedTokenTotalEstimatedAmount": 214897116n,
              },
            ],
            "supportedTokenReceivableAmount": 322345675n,
          },
        },
      }
    `);

    // transfer all vst + enough extra yields to the vault
    await validator.airdropToken(
      knownAddresses.solvProtocolWallet,
      'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
      1_0000_0000n
    );
    expect({
      supportedToken: await ctx.solvProtocolWallet.supportedToken
        .resolve(true)
        .then((a) => a.amount),
      solvReceiptToken: await ctx.solvProtocolWallet.solvReceiptToken
        .resolve(true)
        .then((a) => a.amount),
    }).toMatchInlineSnapshot(`
      {
        "solvReceiptToken": 9999999999n,
        "supportedToken": 400000000n,
      }
    `); // TODO: 9999999999n, where has a penny gone?

    await ctx.donate.execute({
      payer: knownAddresses.solvProtocolWallet,
      supportedTokenAmount: 400000000n,
      receiptTokenAmount: 0n,
      solvReceiptTokenAmount: 0n,
    });

    // cannot process zero amount
    await expect(
      ctx.completeWithdrawalRequests.execute({
        burntSolvReceiptTokenAmount: 0n,
        redeemedSupportedTokenAmount: 0n,
        oldOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_1000_0000_000000n,
      })
    ).rejects.toThrowError();

    // cannot process withdrawals with ambiguous srt amount
    await expect(
      ctx.completeWithdrawalRequests.execute({
        burntSolvReceiptTokenAmount: 10n,
        redeemedSupportedTokenAmount: 11n,
        oldOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_1000_0000_000000n,
      })
    ).rejects.toThrowError();

    await expect(
      ctx.completeWithdrawalRequests.execute({
        burntSolvReceiptTokenAmount: 97680509n - 1n,
        redeemedSupportedTokenAmount: 107448559n,
        oldOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_1000_0000_000000n,
      })
    ).rejects.toThrowError();

    // cannot process withdrawals with not enough vst
    await expect(
      ctx.completeWithdrawalRequests.execute({
        burntSolvReceiptTokenAmount: 97680509n,
        redeemedSupportedTokenAmount: 107448559n - 7448559n,
        oldOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_100_0000_000000n,
      })
    ).rejects.toThrowError();

    // now process 1st req
    await expect(
      ctx.completeWithdrawalRequests.execute({
        burntSolvReceiptTokenAmount: 97680509n,
        redeemedSupportedTokenAmount: 107448559n,
        oldOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_100_0000_000000n,
      })
    ).resolves.not.toThrowError();

    await expect(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "admin": {
          "fundManager": "4pfiCiaZsTf8aNKexuGYzhQLbg6iJKoYospSUmfTpL2K",
          "rewardManager": "5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx",
          "solvManager": "BBiQ99GVfamTcqcYwLgji4k5giL3C8epmR3do1thYigw",
          "vaultManager": "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL",
        },
        "delegatedRewardTokens": [
          {
            "amount": 0n,
            "delegate": {
              "__option": "Some",
              "value": "5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx",
            },
            "mint": "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
          },
        ],
        "oneReceiptTokenAsMicroSupportedTokenAmount": 107448558666666n,
        "oneReceiptTokenAsSupportedTokenAmount": 107448558n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 109999999123456n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 109999999n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 0n,
        "solvProtocolWallet": "4xqLe1ALAh8sbi2N2uEM5JXbhhVNKMVRg3L1m1E2Hfbv",
        "solvProtocolWithdrawalFeeRate": 0.008,
        "solvReceiptTokenAmount": 1n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 400000000n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
        "supportedTokenOperationReservedAmount": 0n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 100000000n,
            "requests": [
              {
                "id": 1n,
                "receiptTokenEnqueuedAmount": 100000000n,
                "solvReceiptTokenLockedAmount": 97680509n,
                "supportedTokenLockedAmount": 0n,
                "supportedTokenTotalEstimatedAmount": 107448559n,
              },
            ],
            "supportedTokenDeductedFeeAmount": 859589n,
            "supportedTokenExtraClaimableAmount": 859589n,
            "supportedTokenTotalClaimableAmount": 107448559n,
          },
          "enqueued": {
            "receiptTokenEnqueuedAmount": 0n,
            "requests": [],
            "solvReceiptTokenLockedAmount": 0n,
            "supportedTokenLockedAmount": 0n,
          },
          "processing": {
            "receiptTokenProcessingAmount": 200000000n,
            "requests": [
              {
                "id": 2n,
                "receiptTokenEnqueuedAmount": 200000000n,
                "solvReceiptTokenLockedAmount": 195361017n,
                "supportedTokenLockedAmount": 0n,
                "supportedTokenTotalEstimatedAmount": 214897116n,
              },
            ],
            "supportedTokenReceivableAmount": 214897116n,
          },
        },
      }
    `);

    // // now process 2nd req
    await expect(
      ctx.completeWithdrawalRequests.execute({
        burntSolvReceiptTokenAmount: 195361017n,
        redeemedSupportedTokenAmount: 214897116n - 1000n,
        oldOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_120_0000_000000n,
      })
    ).resolves.not.toThrowError();

    // withdrawals do not affect redemption rates
    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsMicroSupportedTokenAmount: 107448558666666n,
      oneReceiptTokenAsSupportedTokenAmount: 107448558n,
      oneSolvReceiptTokenAsMicroSupportedTokenAmount: 109999999123456n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 109999999n,
    });
  });
});
