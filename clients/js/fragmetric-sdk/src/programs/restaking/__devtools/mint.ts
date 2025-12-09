import * as token from '@solana-program/token';
import { Address, createNoopSigner, isSome } from '@solana/kit';
import * as v from 'valibot';
import {
  TransactionTemplateContext,
  transformAddressResolverVariant,
} from '../../../context';
import { createMintTransactionTemplate } from '../mint';
import { RestakingProgram } from '../program';

export function createTokenTool(program: RestakingProgram) {
  return {
    createMint: createMintTransactionTemplate(
      program,
      'create a SPL token mint and grant mint authority to admin'
    ),
    mintTo: new TransactionTemplateContext(
      program,
      v.object({
        mint: v.string(),
        recipient: v.string(),
        amount: v.bigint(),
      }),
      {
        description:
          'mint token to arbitrary account (mint authority granted to admin)',
        instructions: [
          async (parent, args, overrides) => {
            const [mint, payer] = await Promise.all([
              token.fetchMint(program.runtime.rpc, args.mint as Address),
              transformAddressResolverVariant(
                overrides.feePayer ??
                  program.runtime.options.transaction.feePayer ??
                  (() => Promise.resolve(null))
              )(program),
            ]);
            if (!(mint && payer)) throw new Error('invalid context');
            const admin = program.knownAddresses.admin;

            const ix =
              await token.getCreateAssociatedTokenIdempotentInstructionAsync({
                payer: createNoopSigner(payer! as Address),
                mint: args.mint as Address,
                owner: args.recipient as Address,
              });
            const recipientTokenAccount = ix.accounts[1].address;

            return Promise.all([
              ix,
              token.getMintToCheckedInstruction({
                mint: mint.address,
                token: recipientTokenAccount,
                mintAuthority: createNoopSigner(
                  isSome(mint.data.mintAuthority)
                    ? mint.data.mintAuthority.value
                    : admin
                ),
                amount: args.amount,
                decimals: mint.data.decimals,
              }),
            ]);
          },
        ],
      }
    ),
    setMintAuthority: new TransactionTemplateContext(
      program,
      v.object({
        mint: v.string(),
        newAuthority: v.string(),
      }),
      {
        description: 'transfer mint authority to input new authority',
        instructions: [
          async (parent, args, overrides) => {
            const [mint, payer] = await Promise.all([
              token.fetchMint(program.runtime.rpc, args.mint as Address),
              transformAddressResolverVariant(
                overrides.feePayer ??
                  program.runtime.options.transaction.feePayer ??
                  (() => Promise.resolve(null))
              )(program),
            ]);
            if (!(mint && payer)) throw new Error('invalid context');

            const ix = token.getSetAuthorityInstruction({
              owned: mint.address,
              owner: isSome(mint.data.mintAuthority)
                ? createNoopSigner(mint.data.mintAuthority.value)
                : ('' as Address),
              authorityType: token.AuthorityType.MintTokens,
              newAuthority: args.newAuthority as Address,
            });
            return Promise.all([ix]);
          },
        ],
      }
    ),
  };
}
