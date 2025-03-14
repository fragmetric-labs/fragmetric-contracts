import { getTransferSolInstruction } from '@solana-program/system';
import { TOKEN_PROGRAM_ADDRESS } from '@solana-program/token';
import { Lamports } from '@solana/kit';
import { LiteSVM } from 'litesvm';
import { describe, expect, test } from 'vitest';
import { ProgramContext } from '../program';
import { createLedgerSignerResolver } from './signer.node';
import { TransactionTemplateContext } from './template';

describe('various signer impl', async () => {
  const program = ProgramContext.connect({
    type: 'litesvm',
    svm: new LiteSVM().withBuiltins().withSysvars().withSplPrograms(),
  });

  const ledgerSignerResolver = createLedgerSignerResolver();

  test.skip('ledger signer', async () => {
    const address = await ledgerSignerResolver().then(
      (signer) => signer.address
    );
    await program.runtime.rpc.requestAirdrop!(
      address,
      1_000_000_000n as Lamports
    ).send();

    const txBuilder1 = new TransactionTemplateContext(program, null, {
      feePayer: ledgerSignerResolver,
      instructions: [
        getTransferSolInstruction({
          amount: 500_000_000n as Lamports,
          destination: TOKEN_PROGRAM_ADDRESS,
          source: address as any,
        }),
      ],
    });

    await expect(txBuilder1.sendAndConfirm(null)).resolves.toBeTypeOf('string');
    await expect(program.runtime.fetchAccount(address)).resolves
      .toMatchInlineSnapshot(`
      {
        "address": "79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84",
        "data": Uint8Array [],
        "executable": false,
        "lamports": 499995000n,
        "programAddress": "11111111111111111111111111111111",
        "space": 0n,
      }
    `);
  });
});
