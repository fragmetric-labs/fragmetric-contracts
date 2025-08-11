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
        "solvProtocolDepositFeeRate": 0.002,
        "solvProtocolWallet": "4xqLe1ALAh8sbi2N2uEM5JXbhhVNKMVRg3L1m1E2Hfbv",
        "solvProtocolWithdrawalFeeRate": 0.006,
        "solvReceiptTokenAmount": 0n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 0n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
        "supportedTokenOperationReceivableAmount": 0n,
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
      supportedTokenOperationReceivableAmount: 446914n,
      solvReceiptTokenAmount: 0n,
      solvReceiptTokenOperationReservedAmount: 0n,
      solvReceiptTokenOperationReceivableAmount: 223009875n,
    });

    await expect(
      ctx.solvProtocolWallet.supportedToken.resolve(true).then((a) => a!.amount)
    ).resolves.toEqual(223456789n);

    // transfer srt to the vault with exact 1:1 rate
    await ctx.donate.execute({
      payer: knownAddresses.solvProtocolWallet,
      supportedTokenAmount: 0n,
      receiptTokenAmount: 0n,
      solvReceiptTokenAmount: 223009875n,
    });

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsSupportedTokenAmount: 1_0000_0000n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 1_0000_0000n,
      receiptTokenSupply: 223456789n,
      supportedTokenAmount: 0n,
      supportedTokenOperationReservedAmount: 0n,
      supportedTokenOperationReceivableAmount: 446914n,
      solvReceiptTokenAmount: 223009875n,
      solvReceiptTokenOperationReservedAmount: 0n,
      solvReceiptTokenOperationReceivableAmount: 223009875n,
    });

    // solv manager cannot complete with less value of srt
    await expect(
      ctx.completeDeposits.execute({
        redeemedSolvReceiptTokenAmount: 10n * 2_2300_9875n,
        newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_000_0000_000000n,
      })
    ).rejects.toThrow();

    // solv manager cannot complete with too rapid srt price incrase
    await expect(
      ctx.completeDeposits.execute({
        redeemedSolvReceiptTokenAmount: (2_2300_9875n * 10n) / 13n,
        newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_3000_0000_000000n,
      })
    ).rejects.toThrow();

    // solv manager can complete with same redemption rate
    await expect(
      ctx.completeDeposits.execute({
        redeemedSolvReceiptTokenAmount: 2_2300_9875n,
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
      supportedTokenOperationReceivableAmount: 446914n,
      solvReceiptTokenAmount: 223009875n,
      solvReceiptTokenOperationReservedAmount: 223009875n,
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
      supportedTokenOperationReceivableAmount: 446914n,
      solvReceiptTokenAmount: 223009875n,
      solvReceiptTokenOperationReservedAmount: 223009875n,
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
      supportedTokenOperationReceivableAmount: 600001n,
      solvReceiptTokenAmount: 223009875n,
      solvReceiptTokenOperationReservedAmount: 223009875n,
      solvReceiptTokenOperationReceivableAmount: 76_390_124n,
    });

    // transfer srt to the vault which is enough to meet exact 1:1.1 rate
    await ctx.donate.execute({
      payer: knownAddresses.solvProtocolWallet,
      supportedTokenAmount: 0n,
      receiptTokenAmount: 0n,
      solvReceiptTokenAmount: 69_435_567n, // 76_390_124n / 1.1 - 10000
    });

    // too less amount with 1:1.05 rate
    await expect(
      ctx.completeDeposits.execute({
        redeemedSolvReceiptTokenAmount: 69_435_567n,
        newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_0500_0000_000000n,
      })
    ).rejects.toThrow();

    // proper amount with near 1:1.1 rate
    await expect(
      ctx.completeDeposits.execute({
        redeemedSolvReceiptTokenAmount: 69_435_567n,
        newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_0999_9999_123456n,
      })
    ).resolves.not.toThrow();

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsSupportedTokenAmount: 107433661n,
      oneReceiptTokenAsMicroSupportedTokenAmount: 107433661666666n,
      oneSolvReceiptTokenAsMicroSupportedTokenAmount: 109999999123456n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 109999999n,
      receiptTokenSupply: 300000000n,
      supportedTokenAmount: 0n,
      supportedTokenOperationReservedAmount: 0n,
      supportedTokenOperationReceivableAmount: 600001n + 11001n, // extra protocol fee
      solvReceiptTokenAmount: 292445442n,
      solvReceiptTokenOperationReservedAmount: 292445442n,
      solvReceiptTokenOperationReceivableAmount: 0n,
    });
    const vaultReceiptTokenValueNumerator = 3_0000_0000n * 1_0743_3661_666666n;
    const vaultNetValueNumerator =
      2_9244_5442n * 1_0999_9999_123456n + 611002n * 1_0000_0000_000000n;
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
      solvReceiptTokenAmount: 292445442n,
      solvReceiptTokenOperationReceivableAmount: 0n,
      solvReceiptTokenOperationReservedAmount: 0n,
      supportedTokenAmount: 0n,
      supportedTokenOperationReservedAmount: 0n,
      supportedTokenOperationReceivableAmount: 0n,
      withdrawal: {
        enqueued: {
          receiptTokenEnqueuedAmount: 300000000n,
          requests: [
            {
              id: 1n,
              receiptTokenEnqueuedAmount: 100000000n,
              solvReceiptTokenLockedAmount: 97481813n,
              supportedTokenLockedAmount: 0n,
              // supportedTokenOffsettedReceivableAmount: 203667n,
              supportedTokenTotalEstimatedAmount: 107433661n,
            },
            {
              id: 2n,
              receiptTokenEnqueuedAmount: 200000000n,
              solvReceiptTokenLockedAmount: 194963629n,
              supportedTokenLockedAmount: 0n,
              // supportedTokenOffsettedReceivableAmount: 407735n,
              supportedTokenTotalEstimatedAmount: 214867324n,
            },
          ],
          solvReceiptTokenLockedAmount: 292445442n,
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
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100000000n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 109999999123456n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 109999999n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 0n,
        "solvProtocolDepositFeeRate": 0.002,
        "solvProtocolWallet": "4xqLe1ALAh8sbi2N2uEM5JXbhhVNKMVRg3L1m1E2Hfbv",
        "solvProtocolWithdrawalFeeRate": 0.006,
        "solvReceiptTokenAmount": 0n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 0n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
        "supportedTokenOperationReceivableAmount": 0n,
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
                "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 109999999123456n,
                "oneSolvReceiptTokenAsSupportedTokenAmount": 109999999n,
                "receiptTokenEnqueuedAmount": 100000000n,
                "solvReceiptTokenLockedAmount": 97481813n,
                "supportedTokenLockedAmount": 0n,
                "supportedTokenTotalEstimatedAmount": 107433661n,
              },
              {
                "id": 2n,
                "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 109999999123456n,
                "oneSolvReceiptTokenAsSupportedTokenAmount": 109999999n,
                "receiptTokenEnqueuedAmount": 200000000n,
                "solvReceiptTokenLockedAmount": 194963629n,
                "supportedTokenLockedAmount": 0n,
                "supportedTokenTotalEstimatedAmount": 214867324n,
              },
            ],
            "supportedTokenReceivableAmount": 322300985n,
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
        .then((a) => a!.amount),
      solvReceiptToken: await ctx.solvProtocolWallet.solvReceiptToken
        .resolve(true)
        .then((a) => a!.amount),
    }).toMatchInlineSnapshot(`
      {
        "solvReceiptToken": 10000000000n,
        "supportedToken": 400000000n,
      }
    `);

    await ctx.donate.execute({
      payer: knownAddresses.solvProtocolWallet,
      supportedTokenAmount: 400000000n,
      receiptTokenAmount: 0n,
      solvReceiptTokenAmount: 0n,
    });

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
        burntSolvReceiptTokenAmount: 97481813n - 1n,
        redeemedSupportedTokenAmount: 106576614n, // 97481813 * 1.1 * (1 - 0.006) - 10000
        oldOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_1000_0000_000000n,
      })
    ).rejects.toThrowError();

    // cannot process withdrawals with not enough vst
    await expect(
      ctx.completeWithdrawalRequests.execute({
        burntSolvReceiptTokenAmount: 97481813n,
        redeemedSupportedTokenAmount: 106576614n - 7448559n,
        oldOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_1000_0000_000000n,
      })
    ).rejects.toThrowError();

    // now process 1st req
    await expect(
      ctx.completeWithdrawalRequests.execute({
        burntSolvReceiptTokenAmount: 97481813n,
        redeemedSupportedTokenAmount: 106576614n,
        oldOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_1000_0000_000000n,
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
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100000000000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100000000n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 109999999123456n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 109999999n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 0n,
        "solvProtocolDepositFeeRate": 0.002,
        "solvProtocolWallet": "4xqLe1ALAh8sbi2N2uEM5JXbhhVNKMVRg3L1m1E2Hfbv",
        "solvProtocolWithdrawalFeeRate": 0.006,
        "solvReceiptTokenAmount": 0n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 400000000n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 0n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 100000000n,
            "requests": [
              {
                "id": 1n,
                "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 109999999123456n,
                "oneSolvReceiptTokenAsSupportedTokenAmount": 109999999n,
                "receiptTokenEnqueuedAmount": 100000000n,
                "solvReceiptTokenLockedAmount": 97481813n,
                "supportedTokenLockedAmount": 0n,
                "supportedTokenTotalEstimatedAmount": 107433661n,
              },
            ],
            "supportedTokenDeductedFeeAmount": 859470n,
            "supportedTokenExtraClaimableAmount": 2423n,
            "supportedTokenTotalClaimableAmount": 106576614n,
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
                "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 109999999123456n,
                "oneSolvReceiptTokenAsSupportedTokenAmount": 109999999n,
                "receiptTokenEnqueuedAmount": 200000000n,
                "solvReceiptTokenLockedAmount": 194963629n,
                "supportedTokenLockedAmount": 0n,
                "supportedTokenTotalEstimatedAmount": 214867324n,
              },
            ],
            "supportedTokenReceivableAmount": 214867324n,
          },
        },
      }
    `);

    // now process 2nd req
    await expect(
      ctx.completeWithdrawalRequests.execute({
        burntSolvReceiptTokenAmount: 194963629n,
        redeemedSupportedTokenAmount: 217049108n - 1000n, // 194963629 * 1.12 * (1 - 0.006) - 1000
        oldOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_1200_0000_000000n,
      })
    ).resolves.not.toThrowError();

    // withdrawals do not affect SRT redemption rates
    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsMicroSupportedTokenAmount: 100000000000000n,
      oneReceiptTokenAsSupportedTokenAmount: 100000000n,
      oneSolvReceiptTokenAsMicroSupportedTokenAmount: 109999999123456n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 109999999n,
    });

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
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 109999999123456n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 109999999n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 0n,
        "solvProtocolDepositFeeRate": 0.002,
        "solvProtocolWallet": "4xqLe1ALAh8sbi2N2uEM5JXbhhVNKMVRg3L1m1E2Hfbv",
        "solvProtocolWithdrawalFeeRate": 0.006,
        "solvReceiptTokenAmount": 0n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 400000000n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
        "supportedTokenOperationReceivableAmount": 0n,
        "supportedTokenOperationReservedAmount": 0n,
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "withdrawal": {
          "completed": {
            "receiptTokenProcessedAmount": 300000000n,
            "requests": [
              {
                "id": 1n,
                "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 109999999123456n,
                "oneSolvReceiptTokenAsSupportedTokenAmount": 109999999n,
                "receiptTokenEnqueuedAmount": 100000000n,
                "solvReceiptTokenLockedAmount": 97481813n,
                "supportedTokenLockedAmount": 0n,
                "supportedTokenTotalEstimatedAmount": 107433661n,
              },
              {
                "id": 2n,
                "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 109999999123456n,
                "oneSolvReceiptTokenAsSupportedTokenAmount": 109999999n,
                "receiptTokenEnqueuedAmount": 200000000n,
                "solvReceiptTokenLockedAmount": 194963629n,
                "supportedTokenLockedAmount": 0n,
                "supportedTokenTotalEstimatedAmount": 214867324n,
              },
            ],
            "supportedTokenDeductedFeeAmount": 2578408n,
            "supportedTokenExtraClaimableAmount": 3902145n,
            "supportedTokenTotalClaimableAmount": 323624722n,
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

    // claim it
    await expect(
      ctx.fundManager.receiptToken.resolve(true).then((a) => a!.amount)
    ).resolves.toMatchInlineSnapshot(`0n`);
    await expect(
      ctx.fundManager.supportedToken.resolve(true).then((a) => a!.amount)
    ).resolves.toMatchInlineSnapshot(`9700000000n`);

    await expect(
      ctx.withdraw.execute({
        payer: knownAddresses.fundManager,
      })
    ).resolves.not.toThrowError();

    await expect(
      ctx.fundManager.supportedToken.resolve(true).then((a) => a!.amount)
    ).resolves.toMatchInlineSnapshot(`10023624722n`);
    // 323624722 = 10023624722 - 9700000000

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
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 109999999123456n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 109999999n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 0n,
        "solvProtocolDepositFeeRate": 0.002,
        "solvProtocolWallet": "4xqLe1ALAh8sbi2N2uEM5JXbhhVNKMVRg3L1m1E2Hfbv",
        "solvProtocolWithdrawalFeeRate": 0.006,
        "solvReceiptTokenAmount": 0n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 0n,
        "supportedTokenAmount": 76375278n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
        "supportedTokenOperationReceivableAmount": 0n,
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

  /** 3. update SRT price */
  test(`solv manager cannot refresh SRT price during deposit`, async () => {
    // fund manager deposits
    await ctx.deposit.execute({
      payer: knownAddresses.fundManager,
      supportedTokenAmount: 1_0000_0000n,
    });

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsMicroSupportedTokenAmount: 100000000000000n,
      oneReceiptTokenAsSupportedTokenAmount: 100000000n,
      oneSolvReceiptTokenAsMicroSupportedTokenAmount: 109999999123456n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 109999999n,
      receiptTokenSupply: 100000000n,
      supportedTokenAmount: 100000000n + 76375278n,
      supportedTokenOperationReservedAmount: 100000000n,
      solvReceiptTokenAmount: 0n,
      solvReceiptTokenOperationReservedAmount: 0n,
      solvReceiptTokenOperationReceivableAmount: 0n,
    });

    // cannot refresh to lower price
    await expect(
      ctx.refreshSolvReceiptTokenRedemptionRate.execute({
        newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_0000_0000_000000n,
      })
    ).rejects.toThrow();

    // solv manager confirms deposit
    await ctx.confirmDeposits.execute(null);

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsMicroSupportedTokenAmount: 100000000000000n,
      oneReceiptTokenAsSupportedTokenAmount: 100000000n,
      oneSolvReceiptTokenAsMicroSupportedTokenAmount: 109999999123456n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 109999999n,
      receiptTokenSupply: 100000000n,
      supportedTokenAmount: 76375278n,
      supportedTokenOperationReservedAmount: 0n,
      supportedTokenOperationReceivableAmount: 200000n,
      solvReceiptTokenAmount: 0n,
      solvReceiptTokenOperationReservedAmount: 0n,
      solvReceiptTokenOperationReceivableAmount: 90727274n,
    });
    // net asset value check
    expect(
      200000n + (90727274n * 109999999123456n) / 100000000000000n
    ).to.equal(100000000n);

    // cannot refresh SRT price during deposit
    await expect(
      ctx.refreshSolvReceiptTokenRedemptionRate.execute({
        newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_1000_0000_000000n,
      })
    ).rejects.toThrow();

    // solv manager completes deposit
    // Redeemed SRT 9071_8182 = (1_0000_0000 * 0.998 - 9999) / 1.1
    await ctx.donate.execute({
      payer: knownAddresses.solvProtocolWallet,
      supportedTokenAmount: 0n,
      receiptTokenAmount: 0n,
      solvReceiptTokenAmount: 9071_8182n,
    });

    await ctx.completeDeposits.execute({
      newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_1000_0000_000000n,
      redeemedSolvReceiptTokenAmount: 9071_8182n,
    });

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsMicroSupportedTokenAmount: 100000000000000n,
      oneReceiptTokenAsSupportedTokenAmount: 100000000n,
      oneSolvReceiptTokenAsMicroSupportedTokenAmount: 110000000000000n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 110000000n,
      receiptTokenSupply: 100000000n,
      supportedTokenAmount: 76375278n,
      supportedTokenOperationReservedAmount: 0n,
      supportedTokenOperationReceivableAmount: 200000n + 10000n,
      solvReceiptTokenAmount: 90718182n,
      solvReceiptTokenOperationReservedAmount: 90718182n,
      solvReceiptTokenOperationReceivableAmount: 0n,
    });
    // net asset value check
    expect(
      200000n + 10000n + (90718182n * 110000000000000n) / 100000000000000n
    ).to.equal(100000000n);

    // after few days SRT redemption rate increased (1%: 1.1 -> 1.111) so solv manager refreshed
    await ctx.refreshSolvReceiptTokenRedemptionRate.execute({
      newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_1110_0000_000000n,
    });

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsMicroSupportedTokenAmount: 100997900000000n,
      oneReceiptTokenAsSupportedTokenAmount: 100997900n,
      oneSolvReceiptTokenAsMicroSupportedTokenAmount: 111100000000000n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 111100000n,
      receiptTokenSupply: 100000000n,
      supportedTokenAmount: 76375278n,
      supportedTokenOperationReservedAmount: 0n,
      supportedTokenOperationReceivableAmount: 200000n + 10000n,
      solvReceiptTokenAmount: 90718182n,
      solvReceiptTokenOperationReservedAmount: 90718182n,
      solvReceiptTokenOperationReceivableAmount: 0n,
    });
    // net asset value check
    expect(
      200000n + 10000n + (90718182n * 111100000000000n) / 100000000000000n
    ).to.equal(100997900n);
  });

  test(`solv manager can adjust SRT price before confirm deposit, but must donate implied fee`, async () => {
    // actually SRT redemption rate increased by 0.1%, not 1%, so current redemption rate is 1.1011, not 1.111
    // so solv manager adjusts redemption rate, and donate implied fee
    await ctx.implySolvProtocolFee.execute({
      newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_1011_0000_000000n,
    });

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsMicroSupportedTokenAmount: 100997900000000n,
      oneReceiptTokenAsSupportedTokenAmount: 100997900n,
      oneSolvReceiptTokenAsMicroSupportedTokenAmount: 110110000000000n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 110110000n,
      receiptTokenSupply: 100000000n,
      supportedTokenAmount: 76375278n,
      supportedTokenOperationReservedAmount: 0n,
      supportedTokenOperationReceivableAmount: 200000n + 10000n + 898110n,
      solvReceiptTokenAmount: 90718182n,
      solvReceiptTokenOperationReservedAmount: 90718182n,
      solvReceiptTokenOperationReceivableAmount: 0n,
    });
    // net asset value check
    expect(
      200000n +
        10000n +
        898110n +
        (90718182n * 110110000000000n) / 100000000000000n
    ).to.equal(100997900n);

    // now donate 0.00888110 VST, which is implied fee due to mistake
    await ctx.confirmDonations.execute({
      redeemedSolvReceiptTokenAmount: 0n,
      redeemedVaultSupportedTokenAmount: 898110n,
    });

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsMicroSupportedTokenAmount: 100997900000000n,
      oneReceiptTokenAsSupportedTokenAmount: 100997900n,
      oneSolvReceiptTokenAsMicroSupportedTokenAmount: 110110000000000n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 110110000n,
      receiptTokenSupply: 100000000n,
      supportedTokenAmount: 76375278n,
      supportedTokenOperationReservedAmount: 898110n,
      supportedTokenOperationReceivableAmount: 200000n + 10000n,
      solvReceiptTokenAmount: 90718182n,
      solvReceiptTokenOperationReservedAmount: 90718182n,
      solvReceiptTokenOperationReceivableAmount: 0n,
    });
    // net asset value check
    expect(
      898110n +
        200000n +
        10000n +
        (90718182n * 110110000000000n) / 100000000000000n
    ).to.equal(100997900n);
  });

  test(`solv manager cannot adjust SRT price during deposit`, async () => {
    // solv manager confirms deposit
    await ctx.confirmDeposits.execute(null);

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsMicroSupportedTokenAmount: 100997900000000n,
      oneReceiptTokenAsSupportedTokenAmount: 100997900n,
      oneSolvReceiptTokenAsMicroSupportedTokenAmount: 110110000000000n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 110110000n,
      receiptTokenSupply: 100000000n,
      supportedTokenAmount: 75477168n,
      supportedTokenOperationReservedAmount: 898110n - 898110n,
      supportedTokenOperationReceivableAmount: 210000n + 1797n,
      solvReceiptTokenAmount: 90718182n,
      solvReceiptTokenOperationReservedAmount: 90718182n,
      solvReceiptTokenOperationReceivableAmount: 814016n,
    });
    // net asset value check
    expect(
      210000n +
        1797n +
        (90718182n * 110110000000000n) / 100000000000000n +
        (814016n * 110110000000000n) / 100000000000000n
    ).to.equal(100997900n);

    // Now, assume that current SRT price is misconfigured: actually 1.08 but set to 1.1011 now.
    // Therefore solv manager wants to lower the price.

    // The current state must have been like:
    // {
    //   oneReceiptTokenAsMicroSupportedTokenAmount: 100997900000000n,
    //   oneReceiptTokenAsSupportedTokenAmount: 100997900n,
    //   oneSolvReceiptTokenAsMicroSupportedTokenAmount: 108000000000000n,
    //   oneSolvReceiptTokenAsSupportedTokenAmount: 108000000n,
    //   receiptTokenSupply: 100000000n,
    //   supportedTokenAmount: 75477168n,
    //   supportedTokenOperationReservedAmount: 0n,
    //   supportedTokenOperationReceivableAmount: 2124154n + 1797n,
    //   solvReceiptTokenAmount: 90718182n,
    //   solvReceiptTokenOperationReservedAmount: 90718182n,
    //   solvReceiptTokenOperationReceivableAmount: 829920n,
    // };
    // net asset value check
    expect(
      2124154n +
        1797n +
        (90718182n * 108000000000000n) / 100000000000000n +
        (829920n * 108000000000000n) / 100000000000000n
    ).to.equal(100997900n);

    // cannot adjust SRT price during deposit
    await expect(
      ctx.implySolvProtocolFee.execute({
        newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_0800_0000_000000n,
      })
    ).rejects.toThrow();

    // Currently SRT receivable is set to 814016 = 898110 * 0.998 / 1.1011
    // In fact, redeemed SRT amount is 820660 = (898110 * 0.998 - 10000) / 1.08
    await ctx.donate.execute({
      payer: knownAddresses.solvProtocolWallet,
      supportedTokenAmount: 0n,
      receiptTokenAmount: 0n,
      solvReceiptTokenAmount: 820660n,
    });

    // With proper SRT price, final state would be:
    const expectedFinalState = {
      oneReceiptTokenAsMicroSupportedTokenAmount: 100997900000000n,
      oneReceiptTokenAsSupportedTokenAmount: 100997900n,
      oneSolvReceiptTokenAsMicroSupportedTokenAmount: 108000000000000n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 108000000n,
      receiptTokenSupply: 100000000n,
      supportedTokenAmount: 75477168n,
      supportedTokenOperationReservedAmount: 0n,
      supportedTokenOperationReceivableAmount: 2124154n + 1797n + 10000n, // 10000 = protocol extra fee,
      solvReceiptTokenAmount: 90718182n + 820660n,
      solvReceiptTokenOperationReservedAmount: 90718182n + 820660n,
      solvReceiptTokenOperationReceivableAmount: 0n,
    };
    // net asset value check
    expect(
      2124154n +
        1797n +
        10000n +
        ((90718182n + 820660n) * 108000000000000n) / 100000000000000n
    ).to.equal(100997900n);

    // To be like expected state, solv manager will
    // 1. complete deposit with current(high) price, less SRT
    // 2. adjust SRT price
    // 3. confirm donation of remaining SRT from step 1

    // 1. complete deposit with current(high) price, less SRT
    await ctx.completeDeposits.execute({
      newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_1011_0000_000000n,
      redeemedSolvReceiptTokenAmount: 814016n,
    });

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsMicroSupportedTokenAmount: 100997900000000n,
      oneReceiptTokenAsSupportedTokenAmount: 100997900n,
      oneSolvReceiptTokenAsMicroSupportedTokenAmount: 110110000000000n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 110110000n,
      receiptTokenSupply: 100000000n,
      supportedTokenAmount: 75477168n,
      supportedTokenOperationReservedAmount: 0n,
      supportedTokenOperationReceivableAmount: 210000n + 1797n,
      solvReceiptTokenAmount: 90718182n + 820660n,
      solvReceiptTokenOperationReservedAmount: 90718182n + 814016n,
      solvReceiptTokenOperationReceivableAmount: 0n,
    });
    // net asset value check
    expect(
      210000n +
        1797n +
        ((90718182n + 814016n) * 110110000000000n) / 100000000000000n
    ).to.equal(100997900n);

    // 2. adjust SRT price
    await ctx.implySolvProtocolFee.execute({
      newOneSolvReceiptTokenAsMicroSupportedTokenAmount: 1_0800_0000_000000n,
    });

    await expect(ctx.resolve(true)).resolves.toMatchObject({
      oneReceiptTokenAsMicroSupportedTokenAmount: 100997900000000n,
      oneReceiptTokenAsSupportedTokenAmount: 100997900n,
      oneSolvReceiptTokenAsMicroSupportedTokenAmount: 108000000000000n,
      oneSolvReceiptTokenAsSupportedTokenAmount: 108000000n,
      receiptTokenSupply: 100000000n,
      supportedTokenAmount: 75477168n,
      supportedTokenOperationReservedAmount: 0n,
      supportedTokenOperationReceivableAmount: 210000n + 1797n + 1931330n,
      solvReceiptTokenAmount: 90718182n + 820660n,
      solvReceiptTokenOperationReservedAmount: 90718182n + 814016n,
      solvReceiptTokenOperationReceivableAmount: 0n,
    });
    // net asset value check
    expect(
      210000n +
        1797n +
        1931330n +
        ((90718182n + 814016n) * 108000000000000n) / 100000000000000n
    ).to.equal(100997900n);

    // 3. donate SRT, and reached expected final state
    await ctx.confirmDonations.execute({
      redeemedSolvReceiptTokenAmount: 6644n, // = 820660 - 814016
      redeemedVaultSupportedTokenAmount: 0n,
    });
    await expect(ctx.resolve(true)).resolves.toMatchObject(expectedFinalState);
  });

  test(`user can deposit srt to mint vrt`, async () => {
    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
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
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100997900000000n,
        "oneReceiptTokenAsSupportedTokenAmount": 100997900n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 108000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 108000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 100000000n,
        "solvProtocolDepositFeeRate": 0.002,
        "solvProtocolWallet": "4xqLe1ALAh8sbi2N2uEM5JXbhhVNKMVRg3L1m1E2Hfbv",
        "solvProtocolWithdrawalFeeRate": 0.006,
        "solvReceiptTokenAmount": 91538842n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 91538842n,
        "supportedTokenAmount": 75477168n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
        "supportedTokenOperationReceivableAmount": 2135951n,
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

    // create user with solv receipt token
    const [signer1] = await Promise.all([
      validator
        .newSigner('SRTDepositTestSigner1', 100_000_000_000n)
        .then(async (signer) => {
          await Promise.all([
            validator.airdropToken(
              signer.address,
              'SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8',
              2000_0000_0000n
            ),
          ]);
          return signer;
        }),
    ]);
    const user1 = ctx.user(signer1);

    // compare expected VRT amount with actually minted amount with random amount of SRT
    for (let i = 1n; i <= 10n; i++) {
      const previousUser1ReceiptTokenAmount = user1.vaultReceiptTokenAccount
        .account
        ? user1.vaultReceiptTokenAccount.account.data.amount
        : 0n;

      const amountToDeposit = 12_3456_7890n * i;

      await ctx.resolve(true);
      const vaultData = ctx.account!.data;

      const vrtSupply = vaultData.vrtSupply;

      const netAssetValueBefore =
        vaultData.vstOperationReservedAmount +
        vaultData.vstOperationReceivableAmount +
        (vaultData.srtOperationReservedAmount * vaultData.oneSrtAsMicroVst) /
          (1_0000_0000n * 1_000_000n) +
        (vaultData.srtOperationReceivableAmount * vaultData.oneSrtAsMicroVst) /
          (1_0000_0000n * 1_000_000n);

      const netAssetValueAfter =
        vaultData.vstOperationReservedAmount +
        vaultData.vstOperationReceivableAmount +
        ((vaultData.srtOperationReservedAmount + amountToDeposit) *
          vaultData.oneSrtAsMicroVst) /
          (1_0000_0000n * 1_000_000n) +
        (vaultData.srtOperationReceivableAmount * vaultData.oneSrtAsMicroVst) /
          (1_0000_0000n * 1_000_000n);

      const expectedVRTAmount =
        ((netAssetValueAfter - netAssetValueBefore) * vrtSupply) /
        netAssetValueBefore;

      const res = await user1.deposit.execute(
        {
          srtAmount: amountToDeposit,
        },
        { signers: [signer1] }
      );

      await user1.resolve(true);
      const currentUser1ReceiptTokenAmount = user1.vaultReceiptTokenAccount
        .account
        ? user1.vaultReceiptTokenAccount.account.data.amount
        : 0n;

      let tokenAmountDiff =
        currentUser1ReceiptTokenAmount - previousUser1ReceiptTokenAmount;

      expect(tokenAmountDiff).toEqual(expectedVRTAmount);
    }

    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
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
        "oneReceiptTokenAsMicroSupportedTokenAmount": 100997900064784n,
        "oneReceiptTokenAsSupportedTokenAmount": 100997900n,
        "oneSolvReceiptTokenAsMicroSupportedTokenAmount": 108000000000000n,
        "oneSolvReceiptTokenAsSupportedTokenAmount": 108000000n,
        "receiptTokenDecimals": 8,
        "receiptTokenMint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 72708769706n,
        "solvProtocolDepositFeeRate": 0.002,
        "solvProtocolWallet": "4xqLe1ALAh8sbi2N2uEM5JXbhhVNKMVRg3L1m1E2Hfbv",
        "solvProtocolWithdrawalFeeRate": 0.006,
        "solvReceiptTokenAmount": 67992772792n,
        "solvReceiptTokenDecimals": 8,
        "solvReceiptTokenMint": "SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8",
        "solvReceiptTokenOperationReceivableAmount": 0n,
        "solvReceiptTokenOperationReservedAmount": 67992772792n,
        "supportedTokenAmount": 75477168n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
        "supportedTokenOperationReceivableAmount": 2135951n,
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
});
