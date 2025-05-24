import * as system from '@solana-program/system';
import * as token from '@solana-program/token';
import {
  Account,
  Address,
  createNoopSigner,
  EncodedAccount,
  None,
} from '@solana/kit';
import * as web3 from '@solana/web3.js';
import * as v from 'valibot';
import {
  AccountContext,
  TransactionTemplateContext,
  transformAddressResolverVariant,
} from '../../context';
import { RestakingFundAccountContext } from './fund';
import { RestakingProgram } from './program';

export class VirtualVaultAccountContext extends AccountContext<
  RestakingFundAccountContext,
  Account<None>
> {
  public resolve(noCache = false, vrtMint?: string) {
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
        const [fund] = await Promise.all([this.parent.resolveAccount(noCache)]);
        if (!fund) {
          return null;
        }
        if (!vrtMint) {
          throw new Error('invalid context: vrt mint should be provided');
        }

        const [virtualVaultAddr] = web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from('virtual_vault'),
            new web3.PublicKey(vrtMint).toBuffer(),
          ],
          new web3.PublicKey(
            this.parent.parent.parent.program.address.toString()
          )
        );
        const vault = fund.data.restakingVaults.filter(
          (restakingVault) =>
            restakingVault.vault == virtualVaultAddr.toString()
        )[0];

        return { vault };
      }
    );
  }

  protected __decodeAccount(
    account: EncodedAccount
  ): Account<Readonly<{ __option: 'None' }>> {
    return {
      address: account.address,
      data: { __option: 'None' },
      executable: account.executable,
      lamports: account.lamports,
      programAddress: account.programAddress,
      space: account.space,
    };
  }

  readonly initializeVrtMint = new TransactionTemplateContext(
    this,
    v.object({
      mint: v.string(),
      name: v.string(),
      symbol: v.string(),
      uri: v.string(),
      description: v.string(),
      decimals: v.number(),
    }),
    {
      description: 'initialize vrt mint',
      instructions: [
        async (parent, args, overrides) => {
          const [payer] = await Promise.all([
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          const admin = (this.program as RestakingProgram).knownAddresses.admin;

          const space = token.getMintSize();
          const rent = await this.runtime.rpc
            .getMinimumBalanceForRentExemption(BigInt(space))
            .send();

          return [
            system.getCreateAccountInstruction({
              payer: createNoopSigner(payer as Address),
              newAccount: createNoopSigner(args.mint as Address),
              lamports: rent,
              space,
              programAddress: token.TOKEN_PROGRAM_ADDRESS,
            }),
            token.getInitializeMintInstruction({
              mint: args.mint as Address,
              decimals: 9,
              mintAuthority: payer as Address,
            }),
            token.getSetAuthorityInstruction({
              owned: args.mint as Address,
              owner: payer as Address,
              authorityType: token.AuthorityType.MintTokens,
              newAuthority: null, // set mint authority to None
            }),
          ];
        },
      ],
    }
  );
}
