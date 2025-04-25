import * as system from '@solana-program/system';
import * as token from '@solana-program/token';
import { Address, createNoopSigner } from '@solana/kit';
import * as v from 'valibot';
import {
  AccountAddressResolverVariant,
  TokenMintAccountContext,
  TransactionTemplateContext,
  transformAddressResolverVariant,
} from '../../context';
import { SolvBTCVaultProgram } from './program';
import { SolvVaultAccountContext } from './vault';

export class SolvVaultReceiptTokenMintAccountContext extends TokenMintAccountContext<SolvBTCVaultProgram> {
  async resolve(noCache = false) {
    // TODO: impl main resolve fn for solv BTC VRT
    const [receiptTokenMint, supportedTokenMint, vault] = await Promise.all([
      this.resolveAddress(),
      this.supportedTokenMint.resolveAddress(),
      this.vault.resolveAddress(),
    ]);
    return {
      receiptTokenMint: receiptTokenMint,
      supportedTokenMint: supportedTokenMint,
      vault: vault,
    };
  }

  constructor(
    parent: SolvBTCVaultProgram,
    addressResolver: AccountAddressResolverVariant<SolvBTCVaultProgram>,
    supportedTokenMint: Address | string
  ) {
    super(parent, addressResolver);
    this.supportedTokenMint =
      new TokenMintAccountContext<SolvVaultReceiptTokenMintAccountContext>(
        this,
        supportedTokenMint
      );
  }

  readonly vault = new SolvVaultAccountContext(this);

  readonly supportedTokenMint: TokenMintAccountContext<SolvVaultReceiptTokenMintAccountContext>;

  /** transactions **/

  readonly initializeMint = new TransactionTemplateContext(
    this,
    v.object({
      authority: v.string(),
    }),
    {
      description: 'initialize receipt token mint',
      instructions: [
        async (parent, args, overrides) => {
          const [receiptTokenMint, payer] = await Promise.all([
            parent.resolveAddress(),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!receiptTokenMint) throw new Error('invalid context');

          const space = token.getMintSize();
          const rent = await this.runtime.rpc
            .getMinimumBalanceForRentExemption(BigInt(space))
            .send();
          return [
            system.getCreateAccountInstruction({
              payer: createNoopSigner(payer! as Address),
              newAccount: createNoopSigner(receiptTokenMint),
              lamports: rent,
              space: space,
              programAddress: token.TOKEN_PROGRAM_ADDRESS,
            }),
            token.getInitializeMint2Instruction({
              mint: receiptTokenMint,
              decimals: 8,
              freezeAuthority: null,
              mintAuthority: args.authority as Address,
            }),
          ];
        },
      ],
    }
  );
}
