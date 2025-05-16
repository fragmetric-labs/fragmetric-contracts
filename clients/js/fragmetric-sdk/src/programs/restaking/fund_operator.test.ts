import { describe, expect, test, vi } from 'vitest';
import { RestakingProgram } from './program';
import { testSignerResolver } from './testing_fixture';

describe('RestakingFundOperatorContext on devnet', async () => {
  const signer = await testSignerResolver();
  const program = RestakingProgram.devnet(process.env.SOLANA_RPC_DEVNET);

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

  test('can execute donateTransaction', async () => {
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

  test('can execute runCommandTransaction (fails as unauthorized)', async () => {
    // const forceResetCommand: restaking.OperationCommandEntryArgs = {
    //   command: {
    //     __kind: 'ProcessWithdrawalBatch',
    //     fields: [{ state: { __kind: 'New' }, forced: true }],
    //   },
    //   requiredAccounts: [],
    // };
    const fnOnSignature = vi.fn();
    const fnOnResult = vi.fn();
    const fnOnError = vi.fn();
    await expect(
      program.fragSOL.fund.runCommand
        .execute(
          {
            forceResetCommand: 'ProcessWithdrawalBatch',
            operator: signer.address,
          },
          {
            feePayer: signer,
            executionHooks: {
              onSignature: fnOnSignature,
              onResult: fnOnResult,
              onError: fnOnError,
            },
          }
        )
        .catch((err) => {
          return err.context.logs.join('\n');
        })
    ).resolves.toContain(
      'Error Code: FundOperationUnauthorizedCommandError. Error Number: 6064. Error Message: fund: unauhorized operation command.'
    );
    expect(fnOnSignature).not.toHaveBeenCalled();
    expect(fnOnResult).not.toHaveBeenCalled();
    expect(fnOnError).toHaveBeenCalledOnce();
  });
});
