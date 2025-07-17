import * as computeBudget from '@solana-program/compute-budget';
import * as system from '@solana-program/system';
import * as token from '@solana-program/token';
import {
  Account,
  Address,
  createNoopSigner,
  EncodedAccount,
} from '@solana/kit';
import * as v from 'valibot';
import {
  AccountAddressResolverVariant,
  AccountContext,
  IterativeAccountContext,
  TokenAccountContext,
  TokenMintAccountContext,
  TransactionTemplateContext,
  transformAddressResolverVariant,
} from '../../context';
import * as solv from '../../generated/solv';
import { FundManagerAccountContext } from './fund_manager';
import { SolvBTCVaultProgram } from './program';
import { SolvProtocolWalletAccountContext } from './solv_protocol_wallet';
import { SolvUserAccountContext } from './user';

export class SolvVaultAccountContext extends AccountContext<
  SolvBTCVaultProgram,
  Account<solv.VaultAccount>
> {
  async resolve(noCache = false) {
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
        const [
          vault,
          receiptTokenMint,
          supportedTokenMint,
          supportedToken,
          solvReceiptToken,
          rewardTokens,
        ] = await Promise.all([
          this.resolveAccount(noCache),
          this.receiptTokenMint.resolveAccount(noCache),
          this.supportedTokenMint.resolveAccount(noCache),
          this.supportedToken.resolveAccount(noCache),
          this.solvReceiptToken.resolveAccount(noCache),
          this.rewardTokens.resolveAccount(noCache),
        ]);
        if (
          !(
            vault &&
            receiptTokenMint &&
            supportedTokenMint &&
            supportedToken &&
            solvReceiptToken
          )
        ) {
          return null;
        }

        const withdrawalRequests = vault.data.withdrawalRequests
          .slice(0, vault.data.numWithdrawalRequests)
          .map((r) => {
            return {
              id: r.requestId,
              receiptTokenEnqueuedAmount: r.vrtWithdrawalRequestedAmount,
              supportedTokenTotalEstimatedAmount:
                r.vstWithdrawalTotalEstimatedAmount,
              supportedTokenLockedAmount: r.vstWithdrawalLockedAmount,
              solvReceiptTokenLockedAmount: r.srtWithdrawalLockedAmount,
              oneSolvReceiptTokenAsSupportedTokenAmount:
                r.oneSrtAsMicroVst / 1_000_000n,
              oneSolvReceiptTokenAsMicroSupportedTokenAmount:
                r.oneSrtAsMicroVst,
              state: r.state,
            };
          });

        return {
          admin: {
            vaultManager: vault.data.vaultManager,
            rewardManager: vault.data.rewardManager,
            fundManager: vault.data.fundManager,
            solvManager: vault.data.solvManager,
          },

          receiptTokenMint: receiptTokenMint.address,
          receiptTokenSupply: receiptTokenMint.data.supply,
          receiptTokenProgram: receiptTokenMint.programAddress,
          receiptTokenDecimals: receiptTokenMint.data.decimals,
          oneReceiptTokenAsSupportedTokenAmount:
            vault.data.oneVrtAsMicroVst / 1_000_000n,
          oneReceiptTokenAsMicroSupportedTokenAmount:
            vault.data.oneVrtAsMicroVst,

          supportedTokenMint: supportedTokenMint.address,
          supportedTokenProgram: supportedTokenMint.programAddress,
          supportedTokenDecimals: supportedTokenMint.data.decimals,
          supportedTokenAmount: supportedToken.data.amount,
          supportedTokenOperationReservedAmount:
            vault.data.vstOperationReservedAmount,
          supportedTokenOperationReceivableAmount:
            vault.data.vstOperationReceivableAmount,

          solvProtocolWallet: vault.data.solvProtocolWallet,
          solvProtocolDepositFeeRate:
            vault.data.solvProtocolDepositFeeRateBps / 10000,
          solvProtocolWithdrawalFeeRate:
            vault.data.solvProtocolWithdrawalFeeRateBps / 10000,
          solvReceiptTokenMint: vault.data.solvReceiptTokenMint,
          solvReceiptTokenDecimals: vault.data.solvReceiptTokenDecimals,
          solvReceiptTokenAmount: solvReceiptToken.data.amount,
          solvReceiptTokenOperationReservedAmount:
            vault.data.srtOperationReservedAmount,
          solvReceiptTokenOperationReceivableAmount:
            vault.data.srtOperationReceivableAmount,
          oneSolvReceiptTokenAsSupportedTokenAmount:
            vault.data.oneSrtAsMicroVst / 1_000_000n,
          oneSolvReceiptTokenAsMicroSupportedTokenAmount:
            vault.data.oneSrtAsMicroVst,

          withdrawal: {
            enqueued: {
              receiptTokenEnqueuedAmount:
                vault.data.vrtWithdrawalEnqueuedAmount,
              supportedTokenLockedAmount: vault.data.vstWithdrawalLockedAmount,
              solvReceiptTokenLockedAmount:
                vault.data.srtWithdrawalLockedAmount,
              requests: withdrawalRequests
                .filter((req) => req.state == 0)
                .map(({ state, ...req }) => req),
            },
            processing: {
              receiptTokenProcessingAmount:
                vault.data.vrtWithdrawalProcessingAmount,
              supportedTokenReceivableAmount:
                vault.data.vstReceivableAmountToClaim,
              requests: withdrawalRequests
                .filter((req) => req.state == 1)
                .map(({ state, ...req }) => req),
            },
            completed: {
              receiptTokenProcessedAmount:
                vault.data.vrtWithdrawalCompletedAmount,
              supportedTokenTotalClaimableAmount:
                vault.data.vstReservedAmountToClaim +
                vault.data.vstExtraAmountToClaim,
              supportedTokenExtraClaimableAmount:
                vault.data.vstExtraAmountToClaim,
              supportedTokenDeductedFeeAmount: vault.data.vstDeductedFeeAmount,
              requests: withdrawalRequests
                .filter((req) => req.state == 2)
                .map(({ state, ...req }) => req),
            },
          },

          delegatedRewardTokens: (rewardTokens ?? [])
            .filter((token) => !!token)
            .map((token) => {
              return {
                mint: token.data.mint,
                amount: token.data.amount,
                delegate: token.data.delegate,
              };
            }),
        };
      }
    );
  }

  protected __decodeAccount(account: EncodedAccount) {
    return solv.decodeVaultAccount(account);
  }

  static fromSeeds(
    parent: SolvBTCVaultProgram,
    seeds: {
      receiptTokenMint: string;
      supportedTokenMint: string;
      solvReceiptTokenMint: string;
    }
  ) {
    return new SolvVaultAccountContext(
      parent,
      async (parent) => {
        const ix =
          await solv.getVaultManagerInitializeVaultAccountInstructionAsync(
            {
              vaultReceiptTokenMint: seeds.receiptTokenMint as Address,
              vaultSupportedTokenMint: seeds.supportedTokenMint as Address,
              solvReceiptTokenMint: seeds.solvReceiptTokenMint as Address,
            } as any,
            { programAddress: parent.program.address }
          );
        return ix.accounts[2].address;
      },
      seeds
    );
  }

  constructor(
    readonly parent: SolvBTCVaultProgram,
    addressResolver: AccountAddressResolverVariant<SolvBTCVaultProgram>,
    seeds: {
      receiptTokenMint: string;
      supportedTokenMint: string;
      solvReceiptTokenMint: string;
    }
  ) {
    super(parent, addressResolver);

    // just for initialization steps
    this.__seedReceiptTokenMint = seeds.receiptTokenMint as Address;
    this.__seedSupportedTokenMint = seeds.supportedTokenMint as Address;
    this.__seedSolvReceiptTokenMint = seeds.solvReceiptTokenMint as Address;
  }
  private readonly __seedReceiptTokenMint: Address;
  private readonly __seedSupportedTokenMint: Address;
  private readonly __seedSolvReceiptTokenMint: Address;

  user(
    addressResolver: AccountAddressResolverVariant<SolvVaultAccountContext>
  ) {
    return new SolvUserAccountContext(this, addressResolver);
  }

  readonly payer = this.user(
    this.runtime.options.transaction.feePayer ?? (() => Promise.resolve(null))
  );

  readonly receiptTokenMint = new TokenMintAccountContext(
    this,
    async (parent) => {
      const vault = await parent.resolveAccount(true);
      if (!vault) return null;
      return vault.data.vaultReceiptTokenMint;
    }
  );

  readonly supportedTokenMint = new TokenMintAccountContext(
    this,
    async (parent) => {
      const vault = await parent.resolveAccount(true);
      if (!vault) return null;
      return vault.data.vaultSupportedTokenMint;
    }
  );

  readonly supportedToken = TokenAccountContext.fromAssociatedTokenSeeds(
    this,
    async (parent) => {
      const vault = await parent.resolveAccount(true);
      if (!vault) return null;
      return {
        owner: vault.address,
        mint: vault.data.vaultSupportedTokenMint,
      };
    }
  );

  readonly solvReceiptTokenMint = new TokenMintAccountContext(
    this,
    async (parent) => {
      const vault = await parent.resolveAccount(true);
      if (!vault) return null;
      return vault.data.solvReceiptTokenMint;
    }
  );

  readonly solvReceiptToken = TokenAccountContext.fromAssociatedTokenSeeds(
    this,
    async (parent) => {
      const vault = await parent.resolveAccount(true);
      if (!vault) return null;
      return {
        owner: vault.address,
        mint: vault.data.solvReceiptTokenMint,
      };
    }
  );

  readonly rewardTokens = new IterativeAccountContext(
    this,
    async (parent) => {
      const vault = await parent.resolveAccount(true);
      if (!vault) return null;
      return (await Promise.all(
        vault.data.delegatedRewardTokenMints
          .slice(0, vault.data.numDelegatedRewardTokenMints)
          .map((item) => {
            return TokenAccountContext.findAssociatedTokenAccountAddress({
              owner: vault.address,
              mint: item,
            });
          })
      )) as Address[];
    },
    async (parent, address) => {
      return new TokenAccountContext(parent, address);
    }
  );

  readonly fundManager = new FundManagerAccountContext(this);

  readonly solvProtocolWallet = new SolvProtocolWalletAccountContext(this);

  /** transactions authorized to vault manager **/
  readonly initializeReceiptTokenMint = new TransactionTemplateContext(
    this,
    null,
    {
      description: 'initialize vault receipt token mint',
      instructions: [
        async (parent, args, overrides) => {
          const [payer] = await Promise.all([
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);

          const vaultManager = (parent.program as SolvBTCVaultProgram)
            .knownAddresses.initialVaultManager;

          const vrtSpace = token.getMintSize();
          const vrtRent = await this.runtime.rpc
            .getMinimumBalanceForRentExemption(BigInt(vrtSpace))
            .send();

          return Promise.all([
            system.getCreateAccountInstruction({
              payer: createNoopSigner(payer! as Address),
              newAccount: createNoopSigner(parent.__seedReceiptTokenMint),
              lamports: vrtRent,
              space: vrtSpace,
              programAddress: token.TOKEN_PROGRAM_ADDRESS,
            }),
            token.getInitializeMint2Instruction({
              mint: parent.__seedReceiptTokenMint,
              decimals: 8,
              freezeAuthority: null,
              mintAuthority: vaultManager,
            }),
          ]);
        },
      ],
    }
  );

  readonly initializeOrUpdateAccount = new TransactionTemplateContext(
    this,
    null,
    {
      description: 'initialize or update vault account',
      instructions: [
        computeBudget.getSetComputeUnitLimitInstruction({ units: 1_400_000 }),
        async (parent, args, overrides) => {
          const [currentVersion, payer] = await Promise.all([
            parent
              .resolveAccount(true)
              .then((vault) => vault?.data.dataVersion ?? 0)
              .catch((err) => 0),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);

          if (currentVersion == 0) {
            const vaultManager = (parent.program as SolvBTCVaultProgram)
              .knownAddresses.initialVaultManager;

            const ix =
              await solv.getVaultManagerInitializeVaultAccountInstructionAsync(
                {
                  payer: createNoopSigner(payer! as Address),
                  vaultManager: createNoopSigner(vaultManager),
                  vaultReceiptTokenMint: parent.__seedReceiptTokenMint,
                  vaultSupportedTokenMint: parent.__seedSupportedTokenMint,
                  solvReceiptTokenMint: parent.__seedSolvReceiptTokenMint,
                  program: this.program.address,
                },
                {
                  programAddress: this.program.address,
                }
              );

            const vault = ix.accounts[2].address;
            return Promise.all([
              token.getCreateAssociatedTokenIdempotentInstructionAsync({
                payer: createNoopSigner(payer as Address),
                mint: parent.__seedReceiptTokenMint,
                owner: vault,
              }),
              token.getCreateAssociatedTokenIdempotentInstructionAsync({
                payer: createNoopSigner(payer as Address),
                mint: parent.__seedSupportedTokenMint,
                owner: vault,
              }),
              token.getCreateAssociatedTokenIdempotentInstructionAsync({
                payer: createNoopSigner(payer as Address),
                mint: parent.__seedSolvReceiptTokenMint,
                owner: vault,
              }),
              ix,
            ]);
          } else {
            const vaultManager = parent.account!.data.vaultManager;
            return Promise.all([
              solv.getVaultManagerUpdateVaultAccountIfNeededInstructionAsync(
                {
                  vaultManager: createNoopSigner(vaultManager),
                  vaultReceiptTokenMint: parent.__seedReceiptTokenMint,
                  vaultSupportedTokenMint: parent.__seedSupportedTokenMint,
                  solvReceiptTokenMint: parent.__seedSolvReceiptTokenMint,
                  program: this.program.address,
                },
                {
                  programAddress: this.program.address,
                }
              ),
            ]);
          }
        },
      ],
    }
  );

  readonly setAdminRoles = new TransactionTemplateContext(
    this,
    v.object({
      vaultManager: v.pipe(
        v.nullable(v.string()),
        v.description('who can manage account data upgrade, i.e. fragBTC admin')
      ),
      rewardManager: v.pipe(
        v.nullable(v.string()),
        v.description('who can delegate rewards, i.e. fragBTC fund manager')
      ),
      fundManager: v.pipe(
        v.nullable(v.string()),
        v.description('who can deposit/withdraw, i.e. fragBTC fund')
      ),
      solvManager: v.pipe(
        v.nullable(v.string()),
        v.description(
          'who can proxy bridging operations, i.e. solv bridge operator'
        )
      ),
    }),
    {
      description: 'initialize vault admin roles',
      instructions: [
        async (parent, args, overrides) => {
          const [vault, payer] = await Promise.all([
            parent.resolveAccount(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);

          if (!vault) {
            throw new Error('invalid context');
          }

          return Promise.all([
            args.vaultManager
              ? solv.getUpdateVaultAdminRoleInstructionAsync(
                  {
                    oldVaultAdmin: createNoopSigner(vault.data.vaultManager),
                    newVaultAdmin: args.vaultManager as Address,
                    vaultReceiptTokenMint: vault.data.vaultReceiptTokenMint,
                    program: this.program.address,
                    role: solv.VaultAdminRole.VaultManager,
                  },
                  {
                    programAddress: this.program.address,
                  }
                )
              : null,
            args.rewardManager
              ? solv.getUpdateVaultAdminRoleInstructionAsync(
                  {
                    oldVaultAdmin: createNoopSigner(vault.data.rewardManager),
                    newVaultAdmin: args.rewardManager as Address,
                    vaultReceiptTokenMint: vault.data.vaultReceiptTokenMint,
                    program: this.program.address,
                    role: solv.VaultAdminRole.RewardManager,
                  },
                  {
                    programAddress: this.program.address,
                  }
                )
              : null,
            ...(args.fundManager
              ? [
                  // to make the fund manager account look alive
                  system.getTransferSolInstruction({
                    source: createNoopSigner(payer as Address),
                    destination: args.fundManager as Address,
                    amount: await this.runtime.rpc
                      .getMinimumBalanceForRentExemption(BigInt(0n))
                      .send(),
                  }),
                  solv.getUpdateVaultAdminRoleInstructionAsync(
                    {
                      oldVaultAdmin: createNoopSigner(vault.data.fundManager),
                      newVaultAdmin: args.fundManager as Address,
                      vaultReceiptTokenMint: vault.data.vaultReceiptTokenMint,
                      program: this.program.address,
                      role: solv.VaultAdminRole.FundManager,
                    },
                    {
                      programAddress: this.program.address,
                    }
                  ),
                ]
              : []),
            args.solvManager
              ? solv.getUpdateVaultAdminRoleInstructionAsync(
                  {
                    oldVaultAdmin: createNoopSigner(vault.data.solvManager),
                    newVaultAdmin: args.solvManager as Address,
                    vaultReceiptTokenMint: vault.data.vaultReceiptTokenMint,
                    program: this.program.address,
                    role: solv.VaultAdminRole.SolvManager,
                  },
                  {
                    programAddress: this.program.address,
                  }
                )
              : null,
          ]);
        },
      ],
    }
  );

  /** transactions authorized to reward manager **/
  readonly delegateRewardTokenAccount = new TransactionTemplateContext(
    this,
    v.object({
      mint: v.string(),
      delegate: v.string(),
    }),
    {
      description: 'delegate reward token mint',
      instructions: [
        async (parent, args, overrides) => {
          const [vault, payer] = await Promise.all([
            parent.resolveAccount(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!vault) throw new Error('invalid context');

          return Promise.all([
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(payer as Address),
              mint: args.mint as Address,
              owner: vault.address,
            }),
            solv.getRewardManagerDelegateRewardTokenAccountInstructionAsync(
              {
                rewardManager: createNoopSigner(
                  vault.data.rewardManager as Address
                ),
                delegate: args.delegate as Address,
                vaultReceiptTokenMint: vault.data.vaultReceiptTokenMint,
                rewardTokenMint: args.mint as Address,
                program: this.program.address,
              },
              {
                programAddress: this.program.address,
              }
            ),
          ]);
        },
      ],
    }
  );

  /** transactions authorized to fund manager **/
  readonly deposit = new TransactionTemplateContext(
    this,
    v.object({
      payer: v.pipe(
        v.string(),
        v.description(
          'ATA of payer will transfer the given amount of supported token'
        )
      ),
      supportedTokenAmount: v.bigint(),
    }),
    {
      description: 'deposit supported token to the vault',
      instructions: [
        async (parent, args, overrides) => {
          const [vault, feePayer] = await Promise.all([
            parent.resolveAccount(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!vault) throw new Error('invalid context');

          return Promise.all([
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(feePayer as Address),
              mint: vault.data.vaultSupportedTokenMint,
              owner: args.payer as Address,
            }),
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(feePayer as Address),
              mint: vault.data.vaultReceiptTokenMint,
              owner: args.payer as Address,
            }),
            solv.getFundManagerDepositInstructionAsync(
              {
                fundManager: createNoopSigner(vault.data.fundManager),
                payer: createNoopSigner(args.payer as Address),
                vaultReceiptTokenMint: vault.data.vaultReceiptTokenMint,
                vaultSupportedTokenMint: vault.data.vaultSupportedTokenMint,
                program: this.program.address,
                vstAmount: args.supportedTokenAmount,
              },
              {
                programAddress: this.program.address,
              }
            ),
          ]);
        },
      ],
    }
  );

  readonly requestWithdrawal = new TransactionTemplateContext(
    this,
    v.object({
      payer: v.pipe(
        v.string(),
        v.description(
          'ATA of payer will transfer the given amount of receipt token'
        )
      ),
      receiptTokenAmount: v.bigint(),
    }),
    {
      description: 'request withdrawal of supported tokens from the vault',
      instructions: [
        async (parent, args, overrides) => {
          const [vault, feePayer] = await Promise.all([
            parent.resolveAccount(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!vault) throw new Error('invalid context');

          return Promise.all([
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(feePayer as Address),
              mint: vault.data.vaultReceiptTokenMint,
              owner: args.payer as Address,
            }),
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(feePayer as Address),
              mint: vault.data.vaultReceiptTokenMint,
              owner: args.payer as Address,
            }),
            solv.getFundManagerRequestWithdrawalInstructionAsync(
              {
                fundManager: createNoopSigner(vault.data.fundManager),
                payer: createNoopSigner(args.payer as Address),
                vaultReceiptTokenMint: vault.data.vaultReceiptTokenMint,
                vaultSupportedTokenMint: vault.data.vaultSupportedTokenMint,
                program: this.program.address,
                vrtAmount: args.receiptTokenAmount,
              },
              {
                programAddress: this.program.address,
              }
            ),
          ]);
        },
      ],
    }
  );

  readonly withdraw = new TransactionTemplateContext(
    this,
    v.object({
      payer: v.pipe(
        v.string(),
        v.description(
          'ATA of payer will recive the claimable amount of supported token'
        )
      ),
    }),
    {
      description:
        'claim supported tokens of completed withdrawals from the vault',
      instructions: [
        async (parent, args, overrides) => {
          const [vault, feePayer] = await Promise.all([
            parent.resolveAccount(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!vault) throw new Error('invalid context');

          return Promise.all([
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(feePayer as Address),
              mint: vault.data.vaultSupportedTokenMint,
              owner: args.payer as Address,
            }),
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(feePayer as Address),
              mint: vault.data.vaultReceiptTokenMint,
              owner: args.payer as Address,
            }),
            solv.getFundManagerWithdrawInstructionAsync(
              {
                fundManager: createNoopSigner(vault.data.fundManager),
                payer: createNoopSigner(args.payer as Address),
                vaultReceiptTokenMint: vault.data.vaultReceiptTokenMint,
                vaultSupportedTokenMint: vault.data.vaultSupportedTokenMint,
                program: this.program.address,
              },
              {
                programAddress: this.program.address,
              }
            ),
          ]);
        },
      ],
    }
  );

  /** transactions authorized to solv manager **/
  readonly confirmDeposits = new TransactionTemplateContext(this, null, {
    description: 'confirm pending deposits toward solv protocol',
    instructions: [
      async (parent, args, overrides) => {
        const [vault, feePayer] = await Promise.all([
          parent.resolveAccount(true),
          transformAddressResolverVariant(
            overrides.feePayer ??
              this.runtime.options.transaction.feePayer ??
              (() => Promise.resolve(null))
          )(parent),
        ]);
        if (!vault) throw new Error('invalid context');

        return Promise.all([
          token.getCreateAssociatedTokenIdempotentInstructionAsync({
            payer: createNoopSigner(feePayer as Address),
            mint: vault.data.vaultSupportedTokenMint,
            owner: vault.data.solvProtocolWallet as Address,
          }),
          token.getCreateAssociatedTokenIdempotentInstructionAsync({
            payer: createNoopSigner(feePayer as Address),
            mint: vault.data.solvReceiptTokenMint,
            owner: vault.data.solvProtocolWallet as Address,
          }),
          solv.getSolvManagerConfirmDepositsInstructionAsync(
            {
              solvManager: createNoopSigner(vault.data.solvManager),
              solvProtocolWallet: vault.data.solvProtocolWallet,
              vaultReceiptTokenMint: vault.data.vaultReceiptTokenMint,
              vaultSupportedTokenMint: vault.data.vaultSupportedTokenMint,
              solvReceiptTokenMint: vault.data.solvReceiptTokenMint,
              program: this.program.address,
            },
            {
              programAddress: this.program.address,
            }
          ),
        ]);
      },
    ],
  });

  readonly completeDeposits = new TransactionTemplateContext(
    this,
    v.object({
      redeemedSolvReceiptTokenAmount: v.pipe(
        v.bigint(),
        v.description(
          'redeemed solv receipt token amount to complete, it can be part of incomplete deposits'
        )
      ),
      newOneSolvReceiptTokenAsMicroSupportedTokenAmount: v.pipe(
        v.bigint(),
        v.description(
          'new redemption rate of srt to vst with +6 more precisions'
        )
      ),
    }),
    {
      description: 'complete deposits toward solv protocol',
      instructions: [
        async (parent, args, overrides) => {
          const [vault, feePayer] = await Promise.all([
            parent.resolveAccount(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!vault) throw new Error('invalid context');

          return Promise.all([
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(feePayer as Address),
              mint: vault.data.vaultSupportedTokenMint,
              owner: vault.data.solvProtocolWallet as Address,
            }),
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(feePayer as Address),
              mint: vault.data.solvReceiptTokenMint,
              owner: vault.data.solvProtocolWallet as Address,
            }),
            solv.getSolvManagerCompleteDepositsInstructionAsync(
              {
                solvManager: createNoopSigner(vault.data.solvManager),
                solvProtocolWallet: vault.data.solvProtocolWallet,
                vaultReceiptTokenMint: vault.data.vaultReceiptTokenMint,
                vaultSupportedTokenMint: vault.data.vaultSupportedTokenMint,
                solvReceiptTokenMint: vault.data.solvReceiptTokenMint,
                program: this.program.address,

                srtAmount: args.redeemedSolvReceiptTokenAmount,
                newOneSrtAsMicroVst:
                  args.newOneSolvReceiptTokenAsMicroSupportedTokenAmount,
              },
              {
                programAddress: this.program.address,
              }
            ),
          ]);
        },
      ],
    }
  );

  readonly refreshSolvReceiptTokenRedemptionRate =
    new TransactionTemplateContext(
      this,
      v.object({
        newOneSolvReceiptTokenAsMicroSupportedTokenAmount: v.pipe(
          v.bigint(),
          v.description(
            'new redemption rate of srt to vst with +6 more precisions'
          )
        ),
      }),
      {
        description:
          'refresh srt redemption rate to update vault total value locked',
        instructions: [
          async (parent, args, overrides) => {
            const [vault, feePayer] = await Promise.all([
              parent.resolveAccount(true),
              transformAddressResolverVariant(
                overrides.feePayer ??
                  this.runtime.options.transaction.feePayer ??
                  (() => Promise.resolve(null))
              )(parent),
            ]);
            if (!vault) throw new Error('invalid context');

            return Promise.all([
              token.getCreateAssociatedTokenIdempotentInstructionAsync({
                payer: createNoopSigner(feePayer as Address),
                mint: vault.data.vaultSupportedTokenMint,
                owner: vault.data.solvProtocolWallet as Address,
              }),
              token.getCreateAssociatedTokenIdempotentInstructionAsync({
                payer: createNoopSigner(feePayer as Address),
                mint: vault.data.solvReceiptTokenMint,
                owner: vault.data.solvProtocolWallet as Address,
              }),
              solv.getSolvManagerRefreshSolvReceiptTokenRedemptionRateInstructionAsync(
                {
                  solvManager: createNoopSigner(vault.data.solvManager),
                  solvProtocolWallet: vault.data.solvProtocolWallet,
                  vaultReceiptTokenMint: vault.data.vaultReceiptTokenMint,
                  vaultSupportedTokenMint: vault.data.vaultSupportedTokenMint,
                  solvReceiptTokenMint: vault.data.solvReceiptTokenMint,
                  program: this.program.address,

                  newOneSrtAsMicroVst:
                    args.newOneSolvReceiptTokenAsMicroSupportedTokenAmount,
                },
                {
                  programAddress: this.program.address,
                }
              ),
            ]);
          },
        ],
      }
    );

  readonly implySolvProtocolFee = new TransactionTemplateContext(
    this,
    v.object({
      newOneSolvReceiptTokenAsMicroSupportedTokenAmount: v.pipe(
        v.bigint(),
        v.description(
          'new redemption rate of srt to vst with +6 more precisions'
        )
      ),
    }),
    {
      description: 'imply solv protocol fee with adjusted srt redemption rate',
      instructions: [
        async (parent, args, overrides) => {
          const [vault, feePayer] = await Promise.all([
            parent.resolveAccount(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!vault) throw new Error('invalid context');

          return Promise.all([
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(feePayer as Address),
              mint: vault.data.vaultSupportedTokenMint,
              owner: vault.data.solvProtocolWallet as Address,
            }),
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(feePayer as Address),
              mint: vault.data.solvReceiptTokenMint,
              owner: vault.data.solvProtocolWallet as Address,
            }),
            solv.getSolvManagerImplySolvProtocolFeeInstructionAsync(
              {
                solvManager: createNoopSigner(vault.data.solvManager),
                solvProtocolWallet: vault.data.solvProtocolWallet,
                vaultReceiptTokenMint: vault.data.vaultReceiptTokenMint,
                vaultSupportedTokenMint: vault.data.vaultSupportedTokenMint,
                solvReceiptTokenMint: vault.data.solvReceiptTokenMint,
                program: this.program.address,

                newOneSrtAsMicroVst:
                  args.newOneSolvReceiptTokenAsMicroSupportedTokenAmount,
              },
              {
                programAddress: this.program.address,
              }
            ),
          ]);
        },
      ],
    }
  );

  readonly confirmDonations = new TransactionTemplateContext(
    this,
    v.object({
      redeemedSolvReceiptTokenAmount: v.pipe(
        v.bigint(),
        v.description('redeemed solv receipt token via donation')
      ),
      redeemedVaultSupportedTokenAmount: v.pipe(
        v.bigint(),
        v.description('redeemed vault supported token via donation')
      ),
    }),
    {
      description: 'confirm donations and offset vst receivables',
      instructions: [
        async (parent, args, overrides) => {
          const [vault, feePayer] = await Promise.all([
            parent.resolveAccount(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!vault) throw new Error('invalid context');

          return Promise.all([
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(feePayer as Address),
              mint: vault.data.vaultSupportedTokenMint,
              owner: vault.data.solvProtocolWallet as Address,
            }),
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(feePayer as Address),
              mint: vault.data.solvReceiptTokenMint,
              owner: vault.data.solvProtocolWallet as Address,
            }),
            solv.getSolvManagerConfirmDonationsInstructionAsync(
              {
                solvManager: createNoopSigner(vault.data.solvManager),
                solvProtocolWallet: vault.data.solvProtocolWallet,
                vaultReceiptTokenMint: vault.data.vaultReceiptTokenMint,
                vaultSupportedTokenMint: vault.data.vaultSupportedTokenMint,
                solvReceiptTokenMint: vault.data.solvReceiptTokenMint,
                program: this.program.address,

                srtAmount: args.redeemedSolvReceiptTokenAmount,
                vstAmount: args.redeemedVaultSupportedTokenAmount,
              },
              {
                programAddress: this.program.address,
              }
            ),
          ]);
        },
      ],
    }
  );

  readonly confirmWithdrawalRequests = new TransactionTemplateContext(
    this,
    null,
    {
      description: 'confirm pending withdrawal requests toward solv protocol',
      instructions: [
        async (parent, args, overrides) => {
          const [vault, feePayer] = await Promise.all([
            parent.resolveAccount(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!vault) throw new Error('invalid context');

          return Promise.all([
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(feePayer as Address),
              mint: vault.data.vaultSupportedTokenMint,
              owner: vault.data.solvProtocolWallet as Address,
            }),
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(feePayer as Address),
              mint: vault.data.solvReceiptTokenMint,
              owner: vault.data.solvProtocolWallet as Address,
            }),
            solv.getSolvManagerConfirmWithdrawalRequestsInstructionAsync(
              {
                solvManager: createNoopSigner(vault.data.solvManager),
                solvProtocolWallet: vault.data.solvProtocolWallet,
                vaultReceiptTokenMint: vault.data.vaultReceiptTokenMint,
                vaultSupportedTokenMint: vault.data.vaultSupportedTokenMint,
                solvReceiptTokenMint: vault.data.solvReceiptTokenMint,
                program: this.program.address,
              },
              {
                programAddress: this.program.address,
              }
            ),
          ]);
        },
      ],
    }
  );

  readonly completeWithdrawalRequests = new TransactionTemplateContext(
    this,
    v.object({
      burntSolvReceiptTokenAmount: v.pipe(
        v.bigint(),
        v.description(
          'burnt solv receipt token amount to complete, it can be part of incomplete withdrawal requests'
        )
      ),
      redeemedSupportedTokenAmount: v.pipe(
        v.bigint(),
        v.description(
          'redeemed vault supported token amount to complete, it can be part of incomplete withdrawal requests'
        )
      ),
      oldOneSolvReceiptTokenAsMicroSupportedTokenAmount: v.pipe(
        v.bigint(),
        v.description(
          'old redemption rate of srt to vst with +6 more precisions'
        )
      ),
    }),
    {
      description: 'complete withdrawal requests toward solv protocol',
      instructions: [
        async (parent, args, overrides) => {
          const [vault, feePayer] = await Promise.all([
            parent.resolveAccount(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!vault) throw new Error('invalid context');

          return Promise.all([
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(feePayer as Address),
              mint: vault.data.vaultSupportedTokenMint,
              owner: vault.data.solvProtocolWallet as Address,
            }),
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(feePayer as Address),
              mint: vault.data.solvReceiptTokenMint,
              owner: vault.data.solvProtocolWallet as Address,
            }),
            solv.getSolvManagerCompleteWithdrawalRequestsInstructionAsync(
              {
                solvManager: createNoopSigner(vault.data.solvManager),
                solvProtocolWallet: vault.data.solvProtocolWallet,
                vaultReceiptTokenMint: vault.data.vaultReceiptTokenMint,
                vaultSupportedTokenMint: vault.data.vaultSupportedTokenMint,
                solvReceiptTokenMint: vault.data.solvReceiptTokenMint,
                program: this.program.address,

                srtAmount: args.burntSolvReceiptTokenAmount,
                vstAmount: args.redeemedSupportedTokenAmount,
                oldOneSrtAsMicroVst:
                  args.oldOneSolvReceiptTokenAsMicroSupportedTokenAmount,
              },
              {
                programAddress: this.program.address,
              }
            ),
          ]);
        },
      ],
    }
  );

  readonly setSolvProtocolWallet = new TransactionTemplateContext(
    this,
    v.object({
      address: v.pipe(v.string(), v.description('cannot be modified once set')),
    }),
    {
      description: 'set solv protocol wallet address',
      instructions: [
        async (parent, args, overrides) => {
          const [vault, payer] = await Promise.all([
            parent.resolveAccount(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!vault) throw new Error('invalid context');

          const rent = await this.runtime.rpc
            .getMinimumBalanceForRentExemption(BigInt(0n))
            .send();
          return Promise.all([
            // to make the solv protocol wallet account look alive
            system.getTransferSolInstruction({
              source: createNoopSigner(payer as Address),
              destination: args.address as Address,
              amount: rent,
            }),
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(payer as Address),
              mint: vault.data.vaultSupportedTokenMint,
              owner: args.address as Address,
            }),
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(payer as Address),
              mint: vault.data.solvReceiptTokenMint,
              owner: args.address as Address,
            }),
            solv.getSolvManagerSetSolvProtocolWalletInstructionAsync(
              {
                solvManager: createNoopSigner(vault.data.solvManager),
                vaultReceiptTokenMint: vault.data.vaultReceiptTokenMint,
                program: this.program.address,
                solvProtocolWallet: args.address as Address,
              },
              {
                programAddress: this.program.address,
              }
            ),
          ]);
        },
      ],
    }
  );

  readonly setSolvProtocolFeeRate = new TransactionTemplateContext(
    this,
    v.object({
      depositFeeRateBps: v.number(),
      withdrawalFeeRateBps: v.number(),
    }),
    {
      description: 'set solv protocol deposit & withdrawal fee rate',
      instructions: [
        async (parent, args, overrides) => {
          const [vault] = await Promise.all([parent.resolveAccount(true)]);
          if (!vault) throw new Error('invalid context');

          return Promise.all([
            solv.getSolvManagerSetSolvProtocolFeeRateInstructionAsync(
              {
                solvManager: createNoopSigner(vault.data.solvManager),
                solvProtocolWallet: vault.data.solvProtocolWallet,
                vaultReceiptTokenMint: vault.data.vaultReceiptTokenMint,
                program: this.program.address,
                depositFeeRateBps: args.depositFeeRateBps,
                withdrawalFeeRateBps: args.withdrawalFeeRateBps,
              },
              {
                programAddress: this.program.address,
              }
            ),
          ]);
        },
      ],
    }
  );

  /** transactions for testing and manual operation **/
  readonly donate = new TransactionTemplateContext(
    this,
    v.object({
      payer: v.pipe(
        v.string(),
        v.description('ATA of payer will transfer the given amount of tokens')
      ),
      receiptTokenAmount: v.nullable(v.bigint(), 0n),
      supportedTokenAmount: v.nullable(v.bigint(), 0n),
      solvReceiptTokenAmount: v.nullable(v.bigint(), 0n),
    }),
    {
      description:
        'transfer arbitrary amount of tokens to the ATAs of the vault',
      instructions: [
        async (parent, args, overrides) => {
          const [vault] = await Promise.all([parent.resolveAccount(true)]);
          if (!vault) throw new Error('invalid context');

          const [
            payerReceiptTokenAccount,
            payerSupportedTokenAccount,
            payerSolvReceiptTokenAccount,
            vaultReceiptTokenAccount,
            vaultSupportedTokenAccount,
            vaultSolvReceiptTokenAccount,
          ] = await Promise.all([
            TokenAccountContext.findAssociatedTokenAccountAddress({
              owner: args.payer as Address,
              mint: vault.data.vaultReceiptTokenMint,
            }),
            TokenAccountContext.findAssociatedTokenAccountAddress({
              owner: args.payer as Address,
              mint: vault.data.vaultSupportedTokenMint,
            }),
            TokenAccountContext.findAssociatedTokenAccountAddress({
              owner: args.payer as Address,
              mint: vault.data.solvReceiptTokenMint,
            }),
            TokenAccountContext.findAssociatedTokenAccountAddress({
              owner: vault.address as Address,
              mint: vault.data.vaultReceiptTokenMint,
            }),
            TokenAccountContext.findAssociatedTokenAccountAddress({
              owner: vault.address as Address,
              mint: vault.data.vaultSupportedTokenMint,
            }),
            TokenAccountContext.findAssociatedTokenAccountAddress({
              owner: vault.address as Address,
              mint: vault.data.solvReceiptTokenMint,
            }),
          ]);

          return Promise.all([
            args.receiptTokenAmount > 0n
              ? token.getTransferCheckedInstruction({
                  mint: vault.data.vaultReceiptTokenMint,
                  decimals: vault.data.vaultReceiptTokenDecimals,
                  authority: createNoopSigner(args.payer as Address),
                  source: payerReceiptTokenAccount,
                  destination: vaultReceiptTokenAccount,
                  amount: args.receiptTokenAmount,
                })
              : null,
            args.supportedTokenAmount > 0n
              ? token.getTransferCheckedInstruction({
                  mint: vault.data.vaultSupportedTokenMint,
                  decimals: vault.data.vaultSupportedTokenDecimals,
                  authority: createNoopSigner(args.payer as Address),
                  source: payerSupportedTokenAccount,
                  destination: vaultSupportedTokenAccount,
                  amount: args.supportedTokenAmount,
                })
              : null,
            args.solvReceiptTokenAmount > 0n
              ? token.getTransferCheckedInstruction({
                  mint: vault.data.solvReceiptTokenMint,
                  decimals: vault.data.solvReceiptTokenDecimals,
                  authority: createNoopSigner(args.payer as Address),
                  source: payerSolvReceiptTokenAccount,
                  destination: vaultSolvReceiptTokenAccount,
                  amount: args.solvReceiptTokenAmount,
                })
              : null,
          ]);
        },
      ],
    }
  );
}
