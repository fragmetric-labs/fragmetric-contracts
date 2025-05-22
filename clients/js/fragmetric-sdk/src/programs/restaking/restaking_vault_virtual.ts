import * as system from '@solana-program/system';
import * as token from '@solana-program/token';
import {
  Account,
  Address,
  createNoopSigner,
  EncodedAccount,
  None,
} from '@solana/kit';
import web3 from '@solana/web3.js';
import * as v from 'valibot';
import {
  AccountContext,
  TokenAccountContext,
  TokenMintAccountContext,
  TransactionTemplateContext,
  transformAddressResolverVariant,
} from '../../context';
import { RestakingFundAccountContext } from './fund';
import { RestakingProgram } from './program';

export class VirtualVaultAccountContext extends AccountContext<
  RestakingFundAccountContext,
  Account<None>
> {
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
        const [fund] = await Promise.all([this.parent.resolveAccount(noCache)]);
        if (!fund) {
          return null;
        }

        const [virtualVaultAddr] = web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from('virtual_vault'),
            new web3.PublicKey(
              this.parent.parent.parent.knownAddresses.fragSOL.toString()
            ).toBuffer(),
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

  readonly initializeVstMint = new TransactionTemplateContext(
    this,
    v.object({
      name: v.string(),
      symbol: v.string(),
      uri: v.string(),
      description: v.string(),
      decimals: v.number(),
    }),
    {
      description: 'initialize vst mint',
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
              newAccount: createNoopSigner(
                'EQFJ1FMNNBeNmznS6QVmMXEZMW55EFXZvzfEepxgXgVE' as Address
              ),
              lamports: rent,
              space,
              programAddress: token.TOKEN_PROGRAM_ADDRESS,
            }),
            token.getInitializeMintInstruction({
              mint: 'EQFJ1FMNNBeNmznS6QVmMXEZMW55EFXZvzfEepxgXgVE' as Address, // vst
              decimals: 9,
              mintAuthority: admin,
            }),
          ];
        },
      ],
    }
  );

  readonly initializeVrtMint = new TransactionTemplateContext(
    this,
    v.object({
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
              newAccount: createNoopSigner(
                '8vEunBQvD3L4aNnRPyQzfQ7pecq4tPb46PjZVKUnTP9i' as Address
              ),
              lamports: rent,
              space,
              programAddress: token.TOKEN_PROGRAM_ADDRESS,
            }),
            token.getInitializeMintInstruction({
              mint: '8vEunBQvD3L4aNnRPyQzfQ7pecq4tPb46PjZVKUnTP9i' as Address, // vrt
              decimals: 9,
              mintAuthority: payer as Address,
            }),
            token.getSetAuthorityInstruction({
              owned: '8vEunBQvD3L4aNnRPyQzfQ7pecq4tPb46PjZVKUnTP9i' as Address, // vrt
              owner: payer as Address,
              authorityType: token.AuthorityType.MintTokens,
              newAuthority: null, // set mint authority to None
            }),
          ];
        },
      ],
    }
  );

  readonly supportedToken = TokenAccountContext.fromAssociatedTokenSeeds(
    this,
    async (parent) => {
      const [fund] = await Promise.all([parent.parent.resolveAccount(true)]);
      if (fund) {
        const [virtualVaultAddr] = web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from('virtual_vault'),
            new web3.PublicKey(
              parent.parent.parent.parent.knownAddresses.fragSOL.toString()
            ).toBuffer(),
          ],
          new web3.PublicKey(
            parent.parent.parent.parent.program.address.toString()
          )
        );
        console.log(`virtualVault address: ${virtualVaultAddr}`);
        const vault = fund.data.restakingVaults.filter(
          (restakingVault) =>
            restakingVault.vault == virtualVaultAddr.toString()
        )[0];
        return {
          owner: virtualVaultAddr.toString(),
          mint: vault.supportedTokenMint,
        };
      }
      return null;
    }
  );

  readonly receiptTokenMint = new TokenMintAccountContext(
    this,
    async (parent) => {
      const [fund] = await Promise.all([parent.parent.resolveAccount(true)]);
      if (fund) {
        const [virtualVaultAddr] = web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from('virtual_vault'),
            new web3.PublicKey(
              parent.parent.parent.parent.knownAddresses.fragSOL.toString()
            ).toBuffer(),
          ],
          new web3.PublicKey(
            parent.parent.parent.parent.program.address.toString()
          )
        );
        const vault = fund.data.restakingVaults.filter(
          (restakingVault) =>
            restakingVault.vault == virtualVaultAddr.toString()
        )[0];
        return vault.receiptTokenMint;
      }
      return null;
    }
  );
}
