import * as token from '@solana-program/token';
import { createNoopSigner } from '@solana/kit';
import * as v from 'valibot';
import {
  BaseAccountContext,
  TokenAccountContext,
  TransactionTemplateContext,
} from '../../context';
import * as solv from '../../generated/solv';
import { SolvVaultAccountContext } from './vault';

export class SolvUserAccountContext extends BaseAccountContext<SolvVaultAccountContext> {
  public resolve(noCache = false) {
    return this.__deduplicated(
      {
        method: 'resolve',
        params: [noCache],
        alternativeParams: noCache ? null : [true],
        intervalSeconds: noCache
          ? 0
          : this.__maybeRuntimeOptions?.rpc.accountDeduplicationIntervalSeconds,
      },
      async () => {
        const user = await this.resolveAccount(noCache);

        if (!user) return null;

        const [userSolvReceiptTokenAccount, userVaultReceiptTokenAccount] =
          await Promise.all([
            this.solvReceiptTokenAccount.resolveAccount(noCache),
            this.vaultReceiptTokenAccount.resolveAccount(noCache),
          ]);

        return {
          user: this.address!,
          solvReceiptTokenAccount: userSolvReceiptTokenAccount?.data.amount,
          vaultReceiptTokenAccount: userVaultReceiptTokenAccount?.data.amount,
        };
      }
    );
  }

  readonly solvReceiptTokenAccount =
    TokenAccountContext.fromAssociatedTokenSeeds(this, async (parent) => {
      const [user, solvReceiptTokenMint] = await Promise.all([
        parent.resolveAddress(),
        parent.parent.solvReceiptTokenMint.resolveAddress(),
      ]);

      if (user && solvReceiptTokenMint) {
        return {
          owner: user,
          mint: solvReceiptTokenMint,
        };
      }

      return null;
    });

  readonly vaultReceiptTokenAccount =
    TokenAccountContext.fromAssociatedTokenSeeds(this, async (parent) => {
      const [user, vaultReceiptTokenMint] = await Promise.all([
        parent.resolveAddress(),
        parent.parent.receiptTokenMint.resolveAddress(),
      ]);

      if (user && vaultReceiptTokenMint) {
        return {
          owner: user,
          mint: vaultReceiptTokenMint,
        };
      }

      return null;
    });

  readonly deposit = new TransactionTemplateContext(
    this,
    v.object({
      srtAmount: v.pipe(v.bigint(), v.description('srt amount to deposit')),
    }),
    {
      description: 'deposit solv receipt tokens to mint vault receipt tokens',
      instructions: [
        async (parent, args) => {
          const [data, user] = await Promise.all([
            parent.parent.resolve(true),
            parent.resolveAddress(),
          ]);

          if (!(data && user)) throw new Error('invalid context');

          return Promise.all([
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(user),
              mint: data.receiptTokenMint,
              owner: user,
              tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
            }),

            (async () => {
              const ix =
                await solv.getUserDepositSolvReceiptTokenInstructionAsync({
                  user: createNoopSigner(user),
                  solvReceiptTokenMint: data.solvReceiptTokenMint,
                  vaultReceiptTokenMint: data.receiptTokenMint,
                  program: this.program.address,
                  srtAmount: args.srtAmount,
                });

              return ix;
            })(),
          ]);
        },
      ],
    }
  );
}
