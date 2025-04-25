import * as token from '@solana-program/token';
import {
  Account,
  Address,
  createNoopSigner,
  EncodedAccount,
  getAddressEncoder,
  getBytesEncoder,
  getProgramDerivedAddress,
} from '@solana/kit';
import * as v from 'valibot';
import {
  AccountAddressResolverVariant,
  AccountContext,
  IterativeAccountContext,
  RuntimeCluster,
  TokenAccountContext,
  TokenMintAccountContext,
  TransactionTemplateContext,
  transformAddressResolverVariant,
} from '../../context';
import * as jitoRestaking from '../../generated/jito_restaking';
import * as jitoVault from '../../generated/jito_vault';
import * as restaking from '../../generated/restaking';
import { RestakingFundAccountContext } from './fund';
import { RestakingProgram } from './program';

export class JitoVaultAccountContext extends AccountContext<
  RestakingFundAccountContext,
  Account<jitoVault.Vault>
> {
  async resolve(noCache = false) {
    const [account, delegations] = await Promise.all([
      this.resolveAccount(noCache),
      this.delegations.resolve(noCache),
    ]);
    if (!account) {
      return null;
    }
    const {
      discriminator,
      base,
      delegationState: { reserved, ...delegationState },
      vaultIndex,
      bump,
      reserved: reserved2,
      ...props
    } = account.data;
    return {
      ...props,
      delegationState,
      delegations,
    };
  }

  static knownAddresses(cluster: RuntimeCluster) {
    return {
      config: 'UwuSgAq4zByffCGCrWH87DsjfsewYjuqHfJEpzw1Jq3' as Address,
      programFeeWallet: (cluster == 'devnet'
        ? '9eZbWiHsPRsxLSiHxzg2pkXsAuQMwAjQrda7C7e21Fw6'
        : '5eosrve6LktMZgVNszYzebgmmC7BjLK8NoWyRQtcmGTF') as Address,
    };
  }

  static knownNCNs(cluster: RuntimeCluster) {
    const map = new Map<Address, string>();
    map.set(
      'jtoF4epChkmd75V2kxXSmywatczAomDqKu6VfWUQocT' as Address,
      'TipRouter'
    );
    map.set(
      (cluster == 'devnet'
        ? 'A9muHr9VqgabHCeEgXyGuAeeAVcW8nLJeHLsmWYGLbv5'
        : 'BGTtt2wdTdhLyFQwSGbNriLZiCxXKBbm29bDvYZ4jD6G') as Address,
      'Switchboard'
    );
    return map;
  }

  readonly knownAddresses = JitoVaultAccountContext.knownAddresses(
    this.runtime.cluster
  );

  readonly knownNCNs = JitoVaultAccountContext.knownNCNs(this.runtime.cluster);

  protected __decodeAccount(account: EncodedAccount) {
    return jitoVault.decodeVault(account);
  }

  readonly receiptTokenMint = new TokenMintAccountContext(
    this,
    async (parent) => {
      const [vault] = await Promise.all([parent.resolveAccount(true)]);
      if (vault) {
        return vault.data.vrtMint;
      }
      return null;
    }
  );

  readonly feeWalletReceiptToken = TokenAccountContext.fromAssociatedTokenSeeds(
    this,
    async (parent) => {
      const [vault] = await Promise.all([parent.resolveAccount(true)]);
      if (vault) {
        return {
          owner: vault.data.feeWallet,
          mint: vault.data.vrtMint,
        };
      }
      return null;
    }
  );

  readonly supportedToken = TokenAccountContext.fromAssociatedTokenSeeds(
    this,
    async (parent) => {
      const [vault] = await Promise.all([parent.resolveAccount(true)]);
      if (vault) {
        return {
          owner: vault.address,
          mint: vault.data.supportedMint,
        };
      }
      return null;
    }
  );

  readonly rewardTokens = new IterativeAccountContext(
    this,
    async (parent) => {
      const [vault, vaultStrategies] = await Promise.all([
        parent.resolveAddress(),
        parent.parent.resolveRestakingVaultStrategies(true),
      ]);
      const vaultStrategy = vaultStrategies?.find(
        (item) => item.vault === vault
      );
      if (!(vault && vaultStrategy)) return null;

      return (await Promise.all(
        vaultStrategy.compoundingRewardTokenMints
          .concat(vaultStrategy.distributingRewardTokenMints)
          .map((mint) => {
            return TokenAccountContext.findAssociatedTokenAccountAddress({
              owner: vault,
              mint: mint,
            });
          })
      )) as Address[];
    },
    async (parent, address) => {
      return new TokenAccountContext(parent, address);
    }
  );

  readonly delegations = new IterativeAccountContext<
    JitoVaultAccountContext,
    JitoVaultDelegationContext
  >(
    this,
    async (parent) => {
      const [vault, vaultStrategies] = await Promise.all([
        parent.resolveAddress(),
        parent.parent.resolveRestakingVaultStrategies(true),
      ]);
      const vaultStrategy = vaultStrategies?.find(
        (item) => item.vault === vault
      );
      if (!(vault && vaultStrategy)) return null;

      return (await Promise.all(
        vaultStrategy.delegations.map(async (delegation) => {
          const ix =
            await restaking.getFundManagerInitializeFundJitoRestakingVaultDelegationInstructionAsync(
              {
                receiptTokenMint: vault,
                vaultAccount: vault,
                operatorAccount: delegation.operator,
                program: this.program.address,
              }
            );
          return ix.accounts[5].address;
        })
      )) as Address[];
    },
    async (parent, address) => {
      return new JitoVaultDelegationContext(parent, address);
    }
  );

  readonly setSecondaryAdmin = new TransactionTemplateContext(
    this,
    v.object({
      admin: v.pipe(
        v.nullish(v.string(), null),
        v.description('set vault admin, default is fund manager')
      ),
      newAdmin: v.string(),
      role: v.enum(jitoVault.VaultAdminRole),
    }),
    {
      description: 'set a new secondary admin with a role',
      instructions: [
        async (parent, args, overrides) => {
          const [vault, payer] = await Promise.all([
            parent.resolveAddress(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!vault) throw new Error('invalid context');
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          return Promise.all([
            jitoVault.getSetSecondaryAdminInstruction({
              config: this.knownAddresses.config,
              vault: vault,
              admin: createNoopSigner((args.admin as Address) ?? fundManager),
              newAdmin: args.newAdmin as Address,
              vaultAdminRole: args.role,
            }),
          ]);
        },
      ],
    }
  );

  readonly delegateRewardTokenAccount = new TransactionTemplateContext(
    this,
    v.object({
      admin: v.pipe(
        v.nullish(v.string(), null),
        v.description('set vault admin, default is fund manager')
      ),
      newDelegate: v.string(),
      mint: v.string(),
      program: v.nullish(v.string(), token.TOKEN_PROGRAM_ADDRESS),
    }),
    {
      description: 'delegate a reward token account',
      instructions: [
        async (parent, args, overrides) => {
          const [vault, payer] = await Promise.all([
            parent.resolveAddress(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!vault) throw new Error('invalid context');
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          return Promise.all([
            jitoVault.getDelegateTokenAccountInstruction({
              config: this.knownAddresses.config,
              vault: vault,
              delegate: args.newDelegate as Address,
              delegateAssetAdmin: createNoopSigner(
                (args.admin as Address) ?? fundManager
              ),
              tokenAccount:
                await TokenAccountContext.findAssociatedTokenAccountAddress({
                  owner: vault,
                  mint: args.mint as Address,
                  tokenProgram: args.program as Address,
                }),
              tokenMint: args.mint as Address,
              tokenProgram: args.program as Address,
            }),
          ]);
        },
      ],
    }
  );
}

export class JitoVaultDelegationContext extends AccountContext<
  JitoVaultAccountContext,
  Account<jitoVault.VaultOperatorDelegation>
> {
  async resolve(noCache = false) {
    const [account, ncns] = await Promise.all([
      this.resolveAccount(noCache),
      this.ncns.resolve(noCache),
    ]);
    if (!account) {
      return null;
    }

    const {
      discriminator,
      vault,
      delegationState: { reserved, ...delegationState },
      bump,
      reserved: reserved2,
      ...props
    } = account.data;
    return {
      ...props,
      delegationState,
      ncns,
    };
  }

  protected __decodeAccount(account: EncodedAccount) {
    return jitoVault.decodeVaultOperatorDelegation(account);
  }

  toContextDescription() {
    const desc = super.toContextDescription();
    const res = {
      ...desc,
      properties: {
        ...desc.properties,
        operator: this.__account?.data.operator,
        index: this.__account?.data.index,
        stakedAmount: this.__account?.data.delegationState.stakedAmount,
        coolingDownAmount:
          this.__account?.data.delegationState.coolingDownAmount,
        enqueuedForCooldownAmount:
          this.__account?.data.delegationState.enqueuedForCooldownAmount,
      },
    };
    return res;
  }

  readonly ncns = new IterativeAccountContext<
    JitoVaultDelegationContext,
    JitoVaultDelegationNCNContext
  >(
    this,
    async (parent) => {
      const delegation = await parent.resolveAccount(true);
      if (!delegation) return null;

      return Promise.all(
        Array.from(parent.parent.knownNCNs.keys()).map((ncn) => {
          return getProgramDerivedAddress({
            programAddress: jitoRestaking.JITO_RESTAKING_PROGRAM_ADDRESS,
            seeds: [
              getBytesEncoder().encode(Buffer.from('ncn_operator_state')),
              getAddressEncoder().encode(ncn),
              getAddressEncoder().encode(delegation.data.operator),
            ],
          }).then(([address, bump]) => `${address}/${ncn}`);
        })
      );
    },
    async (parent, address) => {
      const [ncnOperatorState, ncn] = address.split('/');
      return new JitoVaultDelegationNCNContext(
        parent,
        ncnOperatorState,
        ncn as Address
      );
    }
  );
}

export class JitoVaultDelegationNCNContext extends AccountContext<
  JitoVaultDelegationContext,
  Account<jitoRestaking.NcnOperatorState>
> {
  constructor(
    readonly parent: JitoVaultDelegationContext,
    addressResolver: AccountAddressResolverVariant,
    private readonly __ncn: Address
  ) {
    super(parent, addressResolver);
  }

  async resolve(noCache = false) {
    const account = await this.resolveAccount(noCache);
    if (!account) {
      return null;
    }

    const {
      discriminator,
      ncn,
      operator,
      ncnOptInState,
      operatorOptInState,
      bump,
      reserved,
      ...props
    } = account.data;
    // following Jito's codebase, technically, active or warming up, also inactive or cooling-down
    const ncnActive = ncnOptInState.slotAdded > ncnOptInState.slotRemoved;
    const operatorActive =
      operatorOptInState.slotAdded > operatorOptInState.slotRemoved;
    return {
      name: this.parent.parent.knownNCNs.get(ncn) ?? null,
      active: ncnActive && operatorActive,
      ncn,
      ncnActive,
      operatorActive,
      ...props,
    };
  }

  protected __decodeAccount(account: EncodedAccount) {
    return jitoRestaking.decodeNcnOperatorState(account);
  }

  toContextDescription() {
    const desc = super.toContextDescription();
    const data = this.__account?.data;
    const res = {
      ...desc,
      properties: {
        ...desc.properties,
        name: this.parent.parent.knownNCNs.get(data?.ncn ?? this.__ncn) ?? null,
        active: data
          ? data.ncnOptInState.slotAdded > data.ncnOptInState.slotRemoved &&
            data.operatorOptInState.slotAdded >
              data.operatorOptInState.slotRemoved
          : false,
        ncn: data?.ncn ?? this.__ncn,
        index: data?.index,
      },
    };
    return res;
  }
}
