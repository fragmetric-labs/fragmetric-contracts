import { describe, expect, test, vi } from 'vitest';
import { RestakingProgram } from './program';
import { testSignerResolver } from './testing_fixture';

describe('RestakingFundOperatorContext on devnet', async () => {
  const signer = await testSignerResolver();
  const program = RestakingProgram.devnet();

  test('can execute updatePricesTransaction', async () => {
    await expect(
      program.fragSOL.fund.updatePrices.execute(null, {
        // operator context requires feePayer
        feePayer: signer,
      })
    ).resolves.toMatchObject({
      events: {
        unknown: [],
        operatorUpdatedFundPrices: {
          fundAccount: '4YHmpuyY54Bsj61qNxYGgtQy8xhacfnhdZ6W92rqB64w',
          receiptTokenMint: 'FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo',
        },
      },
      succeeded: true,
    });
  });

  // TODO [sdk]: ix context changed in program but not deployed on devnet yet
  test.skip('can execute donateTransaction', async () => {
    await expect(
      program.fragSOL.fund.donate.execute(
        {
          assetAmount: 100n,
          assetMint: null,
        },
        {
          // operator context requires feePayer
          feePayer: signer,
        },
        {
          skipPreflight: true,
        }
      )
    ).resolves.toMatchObject({
      events: {
        unknown: [],
        operatorDonatedToFund: {
          fundAccount: '4YHmpuyY54Bsj61qNxYGgtQy8xhacfnhdZ6W92rqB64w',
          receiptTokenMint: 'FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo',
          donatedAmount: 100n,
        },
      },
      succeeded: true,
    });
  });

  test('can execute runCommandTransaction', async () => {
    // const forceResetCommand: restaking.OperationCommandEntryArgs = {
    //   command: {
    //     __kind: 'ProcessWithdrawalBatch',
    //     fields: [{ state: { __kind: 'New' }, forced: true }],
    //   },
    //   requiredAccounts: [],
    // };
    const fnOnResult = vi.fn();
    const fnOnError = vi.fn();
    await expect(
      program.fragSOL.fund.runCommand
        .execute(
          {
            forceResetCommand: 'ProcessWithdrawalBatch',
          },
          {
            feePayer: signer,
            executionHooks: {
              onResult: fnOnResult,
              onError: fnOnError,
            },
          },
          {
            skipPreflight: true,
          }
        )
        .then((res) => res.result?.meta?.logMessages ?? [])
    ).resolves.toContain(
      'Program log: AnchorError thrown in programs/restaking/src/lib.rs:644. Error Code: FundOperationUnauthorizedCommandError. Error Number: 6064. Error Message: fund: unauhorized operation command.'
    );
    expect(fnOnResult).toHaveBeenCalledOnce();
    expect(fnOnError).not.toHaveBeenCalled();
  });
});
