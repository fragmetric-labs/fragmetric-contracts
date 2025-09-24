import * as mpl from '@metaplex-foundation/mpl-token-metadata';
import * as umi from '@metaplex-foundation/umi';
import * as umiBundle from '@metaplex-foundation/umi-bundle-defaults';
import * as system from '@solana-program/system';
import * as token from '@solana-program/token';
import {
  AccountRole,
  Address,
  createNoopSigner,
  Instruction,
} from '@solana/kit';
import * as v from 'valibot';
import {
  ProgramDerivedContext,
  TransactionTemplateContext,
  transformAddressResolverVariant,
} from '../../context';
import { RestakingProgram } from './program';

export function createMintTransactionTemplate<
  T extends ProgramDerivedContext<any>,
>(self: T, description: string) {
  return new TransactionTemplateContext(
    self,
    v.object({
      mint: v.string(),
      name: v.string(),
      symbol: v.string(),
      uri: v.string(),
      description: v.string(),
      decimals: v.number(),
    }),
    {
      description: description,
      instructions: [
        async (parent, args, overrides) => {
          const [payer] = await Promise.all([
            transformAddressResolverVariant(
              overrides.feePayer ??
                self.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          const admin = (self.program as RestakingProgram).knownAddresses.admin;

          // build metaplex metadata creation ix
          const umiInstance = umiBundle
            .createUmi('https://api.mainnet-beta.solana.com') // this RPC won't be used
            .use(mpl.mplTokenMetadata());
          umiInstance.use(
            umi.signerIdentity(umi.createNoopSigner(payer as any))
          );

          const ixs: Instruction[] = mpl
            .createV1(umiInstance, {
              mint: umi.createNoopSigner(args.mint as any),
              authority: umi.createNoopSigner(admin as any),
              name: args.name,
              symbol: args.symbol,
              decimals: args.decimals,
              uri: args.uri,
              sellerFeeBasisPoints: umi.percentAmount(0),
              tokenStandard: mpl.TokenStandard.Fungible,
            })
            .getInstructions()
            .map((ix) => {
              return {
                accounts: ix.keys.map((account) => {
                  return {
                    address: account.pubkey as string as Address,
                    role: account.isSigner
                      ? account.isWritable
                        ? AccountRole.WRITABLE_SIGNER
                        : AccountRole.READONLY_SIGNER
                      : account.isWritable
                        ? AccountRole.WRITABLE
                        : AccountRole.READONLY,
                  };
                }),
                data: ix.data,
                programAddress: ix.programId as string as Address,
              };
            });

          const space = token.getMintSize();
          const rent = await self.runtime.rpc
            .getMinimumBalanceForRentExemption(BigInt(space))
            .send();
          return [
            system.getCreateAccountInstruction({
              payer: createNoopSigner(payer! as Address),
              newAccount: createNoopSigner(args.mint as Address),
              lamports: rent,
              space: space,
              programAddress: token.TOKEN_PROGRAM_ADDRESS,
            }),
            token.getInitializeMint2Instruction({
              mint: args.mint as Address,
              decimals: args.decimals,
              freezeAuthority: null,
              mintAuthority: admin,
            }),
            ...ixs,
          ];
        },
      ],
    }
  );
}
