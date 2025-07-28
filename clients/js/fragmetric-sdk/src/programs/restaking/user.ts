import * as computeBudget from '@solana-program/compute-budget';
import * as token from '@solana-program/token';
import * as token2022 from '@solana-program/token-2022';
import {
  AccountRole,
  Address,
  createNoopSigner,
  isSome,
  none,
  some,
} from '@solana/kit';
import * as v from 'valibot';
import {
  BaseAccountContext,
  IterativeAccountContext,
  TokenAccountContext,
  TransactionTemplateContext,
} from '../../context';
import * as restaking from '../../generated/restaking';
import {
  getEd25519Instruction,
  signMessageWithEd25519Keypair,
} from './ed25519';
import { getRestakingAnchorEventDecoders } from './events';
import { RestakingReceiptTokenMintAccountContext } from './receipt_token_mint';
import { RestakingUserFundAccountContext } from './user_fund';
import { RestakingUserRewardAccountContext } from './user_reward';

export class RestakingUserAccountContext extends BaseAccountContext<RestakingReceiptTokenMintAccountContext> {
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
        const [
          fund,
          user,
          userFund,
          userReceiptToken,
          userWrappedToken,
          userSupportedTokens,
        ] = await Promise.all([
          this.parent.fund.resolveAccount(noCache),
          this.resolveAccount(noCache),
          this.fund.resolveAccount(noCache),
          this.receiptToken.resolveAccount(noCache),
          this.wrappedToken.resolveAccount(noCache),
          this.supportedTokens.resolveAccountTree(noCache),
        ]);
        if (!(fund && user)) return null;

        const data = fund.data;
        const supportedTokens = data.supportedTokens.slice(
          0,
          data.numSupportedTokens
        );

        const supportedAssets = [
          {
            mint: null as Address | null,
            program: null as Address | null,
            decimals: 9,
            amount: user.lamports as bigint,
            depositable: !!data.sol.depositable,
            withdrawable: !!data.sol.withdrawable,
            withdrawalPendingBatchId: data.sol.withdrawalPendingBatch.batchId,
            withdrawalLastProcessedBatchId:
              data.sol.withdrawalLastProcessedBatchId,
          },
        ].concat(
          supportedTokens.map((v) => {
            return {
              mint: v.token.tokenMint,
              program: v.token.tokenProgram,
              decimals: v.decimals,
              amount:
                userSupportedTokens?.find((s) => s?.data.mint == v.mint)?.data
                  .amount ?? 0n,
              depositable: !!v.token.depositable,
              withdrawable: !!v.token.withdrawable,
              withdrawalPendingBatchId: v.token.withdrawalPendingBatch.batchId,
              withdrawalLastProcessedBatchId:
                v.token.withdrawalLastProcessedBatchId,
            };
          })
        );

        const withdrawalRequests = (userFund?.data.withdrawalRequests.map(
          (v) => {
            const assetMint = isSome(v.supportedTokenMint)
              ? v.supportedTokenMint.value
              : null;
            const asset = supportedAssets.find((a) => a.mint == assetMint);
            const cancelable =
              asset && asset.withdrawalPendingBatchId == v.batchId;
            const processed =
              !cancelable &&
              asset &&
              asset.withdrawalLastProcessedBatchId >= v.batchId;
            return {
              requestId: v.requestId,
              batchId: v.batchId,
              receiptTokenAmount: v.receiptTokenAmount,
              supportedAssetMint: assetMint as Address | null,
              createdAt: new Date(Number(v.createdAt) * 1000),
              state: cancelable
                ? 'cancelable'
                : processed
                  ? 'claimable'
                  : 'processing',
            };
          }
        ) ?? []) as {
          requestId: bigint;
          batchId: bigint;
          receiptTokenAmount: bigint;
          supportedAssetMint: Address | null;
          createdAt: Date;
          state: 'cancelable' | 'processing' | 'claimable';
        }[];

        const filteredSupportedAssets = supportedAssets
          .filter((a) => a.depositable || a.withdrawable)
          .map((v) => {
            const {
              withdrawalPendingBatchId,
              withdrawalLastProcessedBatchId,
              ...rest
            } = v;
            return rest;
          });

        return {
          user: this.address!,
          lamports: user.lamports,
          receiptTokenMint: fund.data.receiptTokenMint,
          receiptTokenAmount: userReceiptToken?.data?.amount ?? 0n,
          wrappedTokenMint: fund.data.wrappedToken.enabled
            ? fund.data.wrappedToken.mint
            : null,
          wrappedTokenAmount: userWrappedToken?.data?.amount ?? 0n,
          supportedAssets: filteredSupportedAssets,
          maxWithdrawalRequests: 4,
          withdrawalRequests: withdrawalRequests,
        };
      }
    );
  }

  readonly fund = new RestakingUserFundAccountContext(this);

  readonly reward = new RestakingUserRewardAccountContext(this);

  readonly receiptToken = TokenAccountContext.fromAssociatedTokenSeeds2022(
    this,
    async (parent) => {
      const [user, receiptTokenMint] = await Promise.all([
        parent.resolveAddress(),
        parent.parent.resolveAddress(),
      ]);
      if (user && receiptTokenMint) {
        return {
          owner: user,
          mint: receiptTokenMint,
        };
      }
      return null;
    }
  );

  readonly wrappedToken =
    TokenAccountContext.fromAssociatedTokenSeeds<RestakingUserAccountContext>(
      this,
      async (parent) => {
        const [fund, user] = await Promise.all([
          parent.parent.fund.resolveAccount(),
          parent.resolveAddress(),
        ]);
        if (fund?.data.wrappedToken?.enabled && user) {
          return {
            owner: user,
            mint: fund.data.wrappedToken.mint,
            tokenProgram: fund.data.wrappedToken.program,
          };
        }
        return null;
      }
    );

  readonly supportedTokens = new IterativeAccountContext(
    this,
    async (parent) => {
      const [self, fund] = await Promise.all([
        parent.resolveAddress(),
        parent.parent.fund.resolveAccount(),
      ]);
      if (!self || !fund) return null;
      return (
        await Promise.all(
          fund.data.supportedTokens
            .slice(0, fund.data.numSupportedTokens)
            .map((item) => {
              return TokenAccountContext.findAssociatedTokenAccountAddress({
                owner: self,
                mint: item.mint,
                tokenProgram: item.program,
              });
            })
        )
      ).filter((address) => !!address) as Address[];
    },
    async (parent, address) => {
      return new TokenAccountContext(parent, address);
    }
  );

  readonly rewardTokens = new IterativeAccountContext(
    this,
    async (parent) => {
      const [self, globalReward] = await Promise.all([
        parent.resolveAddress(),
        parent.parent.reward.resolve(),
      ]);
      if (!self || !globalReward) return null;
      return (
        await Promise.all(
          globalReward.rewards.map((reward) => {
            return TokenAccountContext.findAssociatedTokenAccountAddress({
              owner: self,
              mint: reward.mint,
              tokenProgram: reward.program,
            });
          })
        )
      ).filter((address) => !!address) as Address[];
    },
    async (parent, address) => {
      return new TokenAccountContext(parent, address);
    }
  );

  private __resolveAddressLookupTable = (parent: this) =>
    parent.parent
      .resolve(true)
      .then((data) => data?.__lookupTableAddress ?? null);

  readonly deposit = new TransactionTemplateContext(
    this,
    v.object({
      assetMint: v.pipe(
        v.nullish(v.string(), null),
        v.description(
          'supported token mint or vault receipt token mint to deposit, null to deposit SOL'
        )
      ),
      assetAmount: v.pipe(
        v.nullish(v.bigint(), null),
        v.description('amount to deposit, null for vault receipt token deposit')
      ),
      metadata: v.pipe(
        v.nullish(
          v.pipe(
            v.object({
              user: v.string(),
              walletProvider: v.string(),
              contributionAccrualRate: v.pipe(
                v.number(),
                v.description('in basis point (1.0 to 100)')
              ),
              expiredAt: v.date(),
              signerKeyPair: v.pipe(
                v.object({
                  privateKey: v.any(),
                  publicKey: v.any(),
                }) as v.GenericSchema<CryptoKeyPair>,
                v.description(
                  'CryptoKeyPair instance of authorized metadata signer'
                )
              ),
            }),
            v.transform((obj) => {
              return {
                ...obj,
                user: obj.user as Address,
                expiredAt: Math.floor(obj.expiredAt.getTime() / 1000),
              };
            })
          ),
          null
        ),
        v.description('extra authorization is required to add deposit metadata')
      ),
      applyPresetComputeUnitLimit: v.pipe(
        v.nullish(v.boolean(), true),
        v.description('apply preset CU limit')
      ),
    }),
    {
      description: 'deposit supported assets to mint receipt token',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'userDepositedToFund',
        'userCreatedOrUpdatedFundAccount',
        'userCreatedOrUpdatedRewardAccount',
        'userDepositedToVault'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args) => {
          const [data, fund, user] = await Promise.all([
            parent.parent.resolve(true),
            parent.parent.fund.resolveAccount(true),
            parent.resolveAddress(),
          ]);
          if (!(data && user)) throw new Error('invalid context');

          return Promise.all([
            args.applyPresetComputeUnitLimit
              ? computeBudget.getSetComputeUnitLimitInstruction({
                  units: 1_400_000,
                })
              : null,
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(user),
              mint: data.receiptTokenMint,
              owner: user,
              tokenProgram: token2022.TOKEN_2022_PROGRAM_ADDRESS,
            }),
            restaking.getUserCreateFundAccountIdempotentInstructionAsync(
              {
                user: createNoopSigner(user),
                program: this.program.address,
                receiptTokenMint: data.receiptTokenMint!,
                desiredAccountSize: null,
              },
              {
                programAddress: this.program.address,
              }
            ),
            restaking.getUserCreateRewardAccountIdempotentInstructionAsync(
              {
                user: createNoopSigner(user),
                program: this.program.address,
                receiptTokenMint: data.receiptTokenMint,
                desiredAccountSize: null,
              },
              {
                programAddress: this.program.address,
              }
            ),

            (async () => {
              // handle deposit metadata here (sig verify)
              if (args.metadata) {
                const { signerKeyPair, ...payload } = args.metadata;
                const message = restaking
                  .getDepositMetadataEncoder()
                  .encode(payload);
                const { publicKey, signature } =
                  await signMessageWithEd25519Keypair(signerKeyPair, message);

                return getEd25519Instruction({
                  publicKey,
                  message,
                  signature,
                });
              }
              return null;
            })(),

            (async () => {
              const ix = await (async function (self) {
                const supportedTokenMints = fund?.data.supportedTokens
                  .slice(0, fund.data.numSupportedTokens)
                  .map((supportedToken) => supportedToken.mint.toString());
                const vaultReceiptTokenMints = fund?.data.restakingVaults
                  .slice(0, fund.data.numRestakingVaults)
                  .map((restakingVault) =>
                    restakingVault.receiptTokenMint.toString()
                  );

                if (
                  args.assetMint !== null &&
                  supportedTokenMints?.includes(args.assetMint)
                ) {
                  return restaking.getUserDepositSupportedTokenInstructionAsync(
                    {
                      user: createNoopSigner(user),
                      receiptTokenMint: data.receiptTokenMint,
                      supportedTokenProgram: token.TOKEN_PROGRAM_ADDRESS,
                      program: self.program.address,
                      userSupportedTokenAccount:
                        await TokenAccountContext.findAssociatedTokenAccountAddress(
                          {
                            owner: user,
                            mint: args.assetMint!,
                            tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
                          }
                        ),
                      supportedTokenMint: args.assetMint as Address,
                      amount: args.assetAmount!,
                      metadata: args.metadata,
                    },
                    {
                      programAddress: self.program.address,
                    }
                  );
                } else if (
                  args.assetMint !== null &&
                  vaultReceiptTokenMints?.includes(args.assetMint)
                ) {
                  if (args.assetAmount !== null) {
                    throw new Error(
                      "Vault receipt token deposit's input amount is not allowed. This always deposits total vault receipt token balance of the account."
                    );
                  }
                  return restaking.getUserDepositVaultReceiptTokenInstructionAsync(
                    {
                      user: createNoopSigner(user),
                      receiptTokenMint: data.receiptTokenMint,
                      vaultReceiptTokenProgram: token.TOKEN_PROGRAM_ADDRESS,
                      program: self.program.address,
                      userVaultReceiptTokenAccount:
                        await TokenAccountContext.findAssociatedTokenAccountAddress(
                          {
                            owner: user,
                            mint: args.assetMint!,
                            tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
                          }
                        ),
                      vaultReceiptTokenMint: args.assetMint as Address,
                      metadata: args.metadata,
                    },
                    {
                      programAddress: self.program.address,
                    }
                  );
                } else if (args.assetMint === null) {
                  return restaking.getUserDepositSolInstructionAsync(
                    {
                      user: createNoopSigner(user),
                      receiptTokenMint: data.receiptTokenMint,
                      program: self.program.address,
                      amount: args.assetAmount!,
                      metadata: args.metadata,
                    },
                    {
                      programAddress: self.program.address,
                    }
                  );
                } else {
                  throw new Error(
                    'input assetMint is not included at both supportedTokenMints and vaultReceiptTokenMints'
                  );
                }
              })(this);

              for (const accountMeta of data.__pricingSources) {
                ix.accounts.push(accountMeta);
              }

              return ix;
            })(),
          ]);
        },
      ],
    }
  );

  readonly requestWithdrawal = new TransactionTemplateContext(
    this,
    v.object({
      assetMint: v.pipe(
        v.nullish(v.string(), null),
        v.description(
          'supported token mint to withdraw in, null to withdraw in SOL'
        )
      ),
      receiptTokenAmount: v.pipe(
        v.bigint(),
        v.description('receipt token amount to withdraw')
      ),
      applyPresetComputeUnitLimit: v.pipe(
        v.nullish(v.boolean(), true),
        v.description('apply preset CU limit')
      ),
    }),
    {
      description:
        'create a withdrawal request to convert receipt tokens back into supported assets',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'userRequestedWithdrawalFromFund',
        'userCreatedOrUpdatedFundAccount',
        'userCreatedOrUpdatedRewardAccount'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args) => {
          const [data, user] = await Promise.all([
            parent.parent.resolve(true),
            parent.resolveAddress(),
          ]);
          if (!(data && user)) throw new Error('invalid context');

          return Promise.all([
            args.applyPresetComputeUnitLimit
              ? computeBudget.getSetComputeUnitLimitInstruction({
                  units: 1_400_000,
                })
              : null,
            restaking.getUserCreateFundAccountIdempotentInstructionAsync(
              {
                user: createNoopSigner(user),
                receiptTokenMint: data.receiptTokenMint!,
                program: this.program.address,
                desiredAccountSize: null,
              },
              {
                programAddress: this.program.address,
              }
            ),
            restaking.getUserCreateRewardAccountIdempotentInstructionAsync(
              {
                user: createNoopSigner(user),
                receiptTokenMint: data.receiptTokenMint,
                program: this.program.address,
                desiredAccountSize: null,
              },
              {
                programAddress: this.program.address,
              }
            ),
            (async () => {
              const ix =
                await restaking.getUserRequestWithdrawalInstructionAsync(
                  {
                    user: createNoopSigner(user),
                    receiptTokenMint: data.receiptTokenMint,
                    program: this.program.address,
                    supportedTokenMint: args.assetMint
                      ? some(args.assetMint as Address)
                      : none(),
                    receiptTokenAmount: args.receiptTokenAmount,
                  },
                  {
                    programAddress: this.program.address,
                  }
                );

              for (const accountMeta of data.__pricingSources) {
                ix.accounts.push(accountMeta);
              }

              return ix;
            })(),
          ]);
        },
      ],
    }
  );

  readonly cancelWithdrawalRequest = new TransactionTemplateContext(
    this,
    v.object({
      assetMint: v.pipe(
        v.nullish(v.string(), null),
        v.description(
          'supported token mint to withdraw in, null to withdraw in SOL'
        )
      ),
      requestId: v.pipe(v.bigint(), v.description('withdrawal request id')),
      applyPresetComputeUnitLimit: v.pipe(
        v.nullish(v.boolean(), true),
        v.description('apply preset CU limit')
      ),
    }),
    {
      description: 'cancel a pending withdrawal request',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'userCanceledWithdrawalRequestFromFund',
        'userCreatedOrUpdatedFundAccount',
        'userCreatedOrUpdatedRewardAccount'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args) => {
          const [data, user] = await Promise.all([
            parent.parent.resolve(true),
            parent.resolveAddress(),
          ]);
          if (!(data && user)) throw new Error('invalid context');

          return Promise.all([
            args.applyPresetComputeUnitLimit
              ? computeBudget.getSetComputeUnitLimitInstruction({
                  units: 1_400_000,
                })
              : null,
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(user),
              mint: data.receiptTokenMint,
              owner: user,
              tokenProgram: token2022.TOKEN_2022_PROGRAM_ADDRESS,
            }),
            restaking.getUserCreateFundAccountIdempotentInstructionAsync(
              {
                user: createNoopSigner(user),
                program: this.program.address,
                receiptTokenMint: data.receiptTokenMint!,
                desiredAccountSize: null,
              },
              {
                programAddress: this.program.address,
              }
            ),
            restaking.getUserCreateRewardAccountIdempotentInstructionAsync(
              {
                user: createNoopSigner(user),
                program: this.program.address,
                receiptTokenMint: data.receiptTokenMint,
                desiredAccountSize: null,
              },
              {
                programAddress: this.program.address,
              }
            ),
            (async () => {
              const ix =
                await restaking.getUserCancelWithdrawalRequestInstructionAsync(
                  {
                    user: createNoopSigner(user),
                    receiptTokenMint: data.receiptTokenMint,
                    program: this.program.address,
                    supportedTokenMint: args.assetMint
                      ? some(args.assetMint as Address)
                      : none(),
                    requestId: args.requestId,
                  },
                  {
                    programAddress: this.program.address,
                  }
                );

              for (const accountMeta of data.__pricingSources) {
                ix.accounts.push(accountMeta);
              }

              return ix;
            })(),
          ]);
        },
      ],
    }
  );

  readonly withdraw = new TransactionTemplateContext(
    this,
    v.object({
      assetMint: v.pipe(
        v.nullish(v.string(), null),
        v.description(
          'supported token mint to withdraw in, null to withdraw in SOL'
        )
      ),
      requestId: v.pipe(v.bigint(), v.description('withdrawal request id')),
      applyPresetComputeUnitLimit: v.pipe(
        v.nullish(v.boolean(), true),
        v.description('apply preset CU limit')
      ),
    }),
    {
      description: 'claim redeemed assets from a processed withdrawal request',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'userWithdrewFromFund',
        'userCreatedOrUpdatedFundAccount',
        'userCreatedOrUpdatedRewardAccount'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args) => {
          const [data, userData] = await Promise.all([
            parent.parent.resolve(true),
            parent.resolve(true),
          ]);
          if (!(data && userData)) throw new Error('invalid context');

          const user = userData.user;
          const request = userData.withdrawalRequests.find(
            (r) =>
              r.supportedAssetMint == args.assetMint &&
              r.requestId == args.requestId
          );
          if (!request) throw new Error('invalid context: request not found');

          return Promise.all([
            args.applyPresetComputeUnitLimit
              ? computeBudget.getSetComputeUnitLimitInstruction({
                  units: 1_400_000,
                })
              : null,
            args.assetMint
              ? token.getCreateAssociatedTokenIdempotentInstructionAsync({
                  payer: createNoopSigner(user),
                  mint: args.assetMint as Address,
                  owner: user,
                })
              : null,
            restaking.getUserCreateFundAccountIdempotentInstructionAsync(
              {
                user: createNoopSigner(user),
                receiptTokenMint: data.receiptTokenMint!,
                program: this.program.address,
                desiredAccountSize: null,
              },
              {
                programAddress: this.program.address,
              }
            ),
            restaking.getUserCreateRewardAccountIdempotentInstructionAsync(
              {
                user: createNoopSigner(user),
                receiptTokenMint: data.receiptTokenMint,
                program: this.program.address,
                desiredAccountSize: null,
              },
              {
                programAddress: this.program.address,
              }
            ),
            (async () => {
              const ix = await (args.assetMint
                ? restaking.getUserWithdrawSupportedTokenInstructionAsync(
                    {
                      user: createNoopSigner(user),
                      receiptTokenMint: data.receiptTokenMint,
                      program: this.program.address,
                      batchId: request?.batchId ?? 0n,
                      requestId: args.requestId,
                      supportedTokenMint: args.assetMint as Address,
                      supportedTokenProgram: token.TOKEN_PROGRAM_ADDRESS,
                      userSupportedTokenAccount:
                        await TokenAccountContext.findAssociatedTokenAccountAddress(
                          {
                            owner: user,
                            mint: args.assetMint,
                            tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
                          }
                        ),
                    },
                    {
                      programAddress: this.program.address,
                    }
                  )
                : restaking.getUserWithdrawSolInstructionAsync(
                    {
                      user: createNoopSigner(user),
                      receiptTokenMint: data.receiptTokenMint,
                      program: this.program.address,
                      batchId: request?.batchId ?? 0n,
                      requestId: args.requestId,
                    },
                    {
                      programAddress: this.program.address,
                    }
                  ));

              return ix;
            })(),
          ]);
        },
      ],
    }
  );

  readonly wrap = new TransactionTemplateContext(
    this,
    v.object({
      receiptTokenAmount: v.pipe(
        v.bigint(),
        v.description('receipt token amount to wrap')
      ),
      receiptTokenAmountAsTargetBalance: v.pipe(
        v.nullish(v.boolean(), false),
        v.description(
          'to use receiptTokenAmount as the desired target balance and wraps only the lacking amount'
        )
      ),
      applyPresetComputeUnitLimit: v.pipe(
        v.nullish(v.boolean(), true),
        v.description('apply preset CU limit')
      ),
    }),
    {
      description: 'convert receipt tokens into wrapped tokens',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'userWrappedReceiptToken'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args) => {
          const [data, user] = await Promise.all([
            parent.parent.resolve(true),
            parent.resolveAddress(),
          ]);
          if (!(data && data.wrappedTokenMint && user))
            throw new Error('invalid context');

          return Promise.all([
            args.applyPresetComputeUnitLimit
              ? computeBudget.getSetComputeUnitLimitInstruction({
                  units: 1_400_000,
                })
              : null,
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(user),
              mint: data.wrappedTokenMint,
              owner: user,
            }),
            args.receiptTokenAmountAsTargetBalance
              ? restaking.getUserWrapReceiptTokenIfNeededInstructionAsync(
                  {
                    user: createNoopSigner(user),
                    receiptTokenMint: data.receiptTokenMint,
                    wrappedTokenMint: data.wrappedTokenMint as Address,
                    program: this.program.address,
                    targetBalance: args.receiptTokenAmount,
                  },
                  {
                    programAddress: this.program.address,
                  }
                )
              : restaking.getUserWrapReceiptTokenInstructionAsync(
                  {
                    user: createNoopSigner(user),
                    receiptTokenMint: data.receiptTokenMint,
                    wrappedTokenMint: data.wrappedTokenMint as Address,
                    program: this.program.address,
                    amount: args.receiptTokenAmount,
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

  readonly unwrap = new TransactionTemplateContext(
    this,
    v.object({
      wrappedTokenAmount: v.pipe(
        v.bigint(),
        v.description('wrapped token amount to unwrap')
      ),
      applyPresetComputeUnitLimit: v.pipe(
        v.nullish(v.boolean(), true),
        v.description('apply preset CU limit')
      ),
    }),
    {
      description: 'convert wrapped tokens back into receipt tokens',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'userUnwrappedReceiptToken'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args) => {
          const [data, user] = await Promise.all([
            parent.parent.resolve(true),
            parent.resolveAddress(),
          ]);
          if (!(data && data.wrappedTokenMint && user))
            throw new Error('invalid context');

          return Promise.all([
            args.applyPresetComputeUnitLimit
              ? computeBudget.getSetComputeUnitLimitInstruction({
                  units: 1_400_000,
                })
              : null,
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(user),
              mint: data.receiptTokenMint,
              owner: user,
              tokenProgram: token2022.TOKEN_2022_PROGRAM_ADDRESS,
            }),
            restaking.getUserUnwrapReceiptTokenInstructionAsync(
              {
                user: createNoopSigner(user),
                receiptTokenMint: data.receiptTokenMint,
                wrappedTokenMint: data.wrappedTokenMint as Address,
                program: this.program.address,
                amount: args.wrappedTokenAmount,
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

  /** TODO [sdk]: need to support @solana/web3.js for Wallet integration with transfer hook.
   /* ref: https://github.com/solana-program/token-2022/pull/212#issuecomment-2675360222
   */
  readonly transfer = new TransactionTemplateContext(
    this,
    v.object({
      receiptTokenAmount: v.pipe(
        v.bigint(),
        v.description(
          'receipt token amount to transfer (denominated in smallest units)'
        )
      ),
      recipient: v.pipe(
        v.string(),
        v.description('recipient address to derive destination token account')
      ),
      applyPresetComputeUnitLimit: v.pipe(
        v.nullish(v.boolean(), true),
        v.description('apply preset CU limit')
      ),
    }),
    {
      description: 'transfer receipt token',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'userTransferredReceiptToken'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args, overrides) => {
          const [receiptTokenMint, user] = await Promise.all([
            parent.parent.resolveAccount(true),
            parent.resolveAddress(),
          ]);
          if (!(receiptTokenMint && user)) throw new Error('invalid context');

          // Note that @solana/kit doesn't provide utility fn to resolve extra accounts required for transfer hook for now.
          // just manually added extra meta here based on `fn receipt_token_extra_account_metas() in restaking/src/modules/fund/fund_receipt_token_configuration_service.rs`.
          // ref (web3.js): https://github.com/solana-program/token-2022/blob/main/clients/js-legacy/src/extensions/transferHook/instructions.ts#L189
          // below ixs are just used to calculate accounts.
          const [ixSrc, ixDst, ixHook] = await Promise.all([
            restaking.getUserDepositSolInstructionAsync(
              {
                user: createNoopSigner(user),
                receiptTokenMint: receiptTokenMint.address,
                program: this.program.address,
                amount: 0,
                metadata: null,
              },
              {
                programAddress: this.program.address,
              }
            ),
            restaking.getUserDepositSolInstructionAsync(
              {
                user: createNoopSigner(args.recipient as Address),
                receiptTokenMint: receiptTokenMint.address,
                program: this.program.address,
                amount: 0,
                metadata: null,
              },
              {
                programAddress: this.program.address,
              }
            ),
            restaking.getTokenTransferHookInstructionAsync(
              {
                owner: user as Address,
                sourceReceiptTokenAccount: user, // not used here
                destinationReceiptTokenAccount: user, // not used here
                receiptTokenMint: receiptTokenMint.address,
                amount: 0,
              },
              {
                programAddress: this.program.address,
              }
            ),
          ]);
          const src = ixSrc.accounts[5].address;
          const fund = ixSrc.accounts[6].address;
          const srcFund = ixSrc.accounts[8].address;
          const reward = ixSrc.accounts[9].address;
          const srcReward = ixSrc.accounts[10].address;
          const eventAuthority = ixSrc.accounts[12].address;
          const dst = ixDst.accounts[5].address;
          const dstFund = ixDst.accounts[8].address;
          const dstReward = ixDst.accounts[10].address;
          const extraAccountMetaList = ixHook.accounts[4].address;
          const extraAccounts = [
            [fund, true],
            [reward, true],
            [srcFund, true],
            [srcReward, true],
            [dstFund, true],
            [dstReward, true],
            [eventAuthority, false],
            [this.program.address, false],
            [extraAccountMetaList, false],
            [this.program.address, false],
          ] as const;

          const transferIx = token2022.getTransferCheckedInstruction({
            amount: args.receiptTokenAmount,
            decimals: receiptTokenMint.data.decimals,
            authority: createNoopSigner(user),
            source: src,
            destination: dst,
            mint: receiptTokenMint.address,
          });
          for (const [address, writable] of extraAccounts) {
            transferIx.accounts.push({
              role: writable ? AccountRole.WRITABLE : AccountRole.READONLY,
              address,
            });
          }

          return Promise.all([
            args.applyPresetComputeUnitLimit
              ? computeBudget.getSetComputeUnitLimitInstruction({
                  units: 1_400_000,
                })
              : null,
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(user),
              mint: receiptTokenMint.address,
              owner: args.recipient as Address,
              tokenProgram: token2022.TOKEN_2022_PROGRAM_ADDRESS,
            }),
            transferIx,
          ]);
        },
      ],
    }
  );
}
